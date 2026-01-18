import type { DouxEvent, SoundInfo, ClockMessage, DouxOptions, PreparedMessage } from './types';

const soundMap = new Map<string, string[]>();
const loadedSounds = new Map<string, SoundInfo>();
const loadingSounds = new Map<string, Promise<SoundInfo>>();
let pcm_offset = 0;

const sources = [
	'triangle', 'tri', 'sine', 'sawtooth', 'saw', 'zawtooth', 'zaw',
	'pulse', 'square', 'pulze', 'zquare', 'white', 'pink', 'brown',
	'live', 'livein', 'mic'
];

function githubPath(base: string, subpath = ''): string {
	if (!base.startsWith('github:')) {
		throw new Error('expected "github:" at the start of pseudoUrl');
	}
	let [, path] = base.split('github:');
	path = path.endsWith('/') ? path.slice(0, -1) : path;
	if (path.split('/').length === 2) {
		path += '/main';
	}
	return `https://raw.githubusercontent.com/${path}/${subpath}`;
}

async function fetchSampleMap(url: string): Promise<[Record<string, string[]>, string] | undefined> {
	if (url.startsWith('github:')) {
		url = githubPath(url, 'strudel.json');
	}
	if (url.startsWith('local:')) {
		url = `http://localhost:5432`;
	}
	if (url.startsWith('shabda:')) {
		const [, path] = url.split('shabda:');
		url = `https://shabda.ndre.gr/${path}.json?strudel=1`;
	}
	if (url.startsWith('shabda/speech')) {
		let [, path] = url.split('shabda/speech');
		path = path.startsWith('/') ? path.substring(1) : path;
		const [params, words] = path.split(':');
		let gender = 'f';
		let language = 'en-GB';
		if (params) {
			[language, gender] = params.split('/');
		}
		url = `https://shabda.ndre.gr/speech/${words}.json?gender=${gender}&language=${language}&strudel=1'`;
	}
	if (typeof fetch !== 'function') {
		return;
	}
	const base = url.split('/').slice(0, -1).join('/');
	const json = await fetch(url)
		.then((res) => {
			if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
			return res.json();
		})
		.catch((error) => {
			throw new Error(`error loading "${url}": ${error.message}`);
		});
	return [json, json._base || base];
}

export async function douxsamples(
	sampleMap: string | Record<string, string[]>,
	baseUrl?: string
): Promise<void> {
	if (typeof sampleMap === 'string') {
		const result = await fetchSampleMap(sampleMap);
		if (!result) return;
		const [json, base] = result;
		return douxsamples(json, base);
	}
	Object.entries(sampleMap).map(async ([key, urls]) => {
		if (key !== '_base') {
			urls = urls.map((url) => baseUrl + url);
			soundMap.set(key, urls);
		}
	});
}

const BLOCK_SIZE = 128;
const CHANNELS = 2;
const CLOCK_SIZE = 16;

// AudioWorklet processor code - runs in worklet context
const workletCode = `
const BLOCK_SIZE = 128;
const CHANNELS = 2;
const CLOCK_SIZE = 16;

let wasmExports = null;
let wasmMemory = null;
let output = null;
let input_buffer = null;
let event_input_ptr = 0;
let framebuffer = null;
let framebuffer_ptr = 0;
let frame_ptr = 0;
let frameIdx = 0;
let block = 0;

class DouxProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super(options);
    this.active = true;
    this.clock_active = options.processorOptions?.clock_active || false;
    this.clockmsg = {
      clock: true,
      t0: 0,
      t1: 0,
      latency: (CLOCK_SIZE * BLOCK_SIZE) / sampleRate,
    };
    this.port.onmessage = async (e) => {
      const { wasm, evaluate, event_input, panic, writePcm } = e.data;
      if (wasm) {
        const { instance } = await WebAssembly.instantiate(wasm, {});
        wasmExports = instance.exports;
        wasmMemory = wasmExports.memory;
        wasmExports.doux_init(sampleRate);
        event_input_ptr = wasmExports.get_event_input_pointer();
        output = new Float32Array(
          wasmMemory.buffer,
          wasmExports.get_output_pointer(),
          BLOCK_SIZE * CHANNELS,
        );
        input_buffer = new Float32Array(
          wasmMemory.buffer,
          wasmExports.get_input_buffer_pointer(),
          BLOCK_SIZE * CHANNELS,
        );
        framebuffer_ptr = wasmExports.get_framebuffer_pointer();
        frame_ptr = wasmExports.get_frame_pointer();
        const framebufferLen = Math.floor((sampleRate / 60) * CHANNELS) * 4;
        framebuffer = new Float32Array(framebufferLen);
        this.port.postMessage({ ready: true, sampleRate });
      } else if (writePcm) {
        const { data, offset } = writePcm;
        const pcm_ptr = wasmExports.get_sample_buffer_pointer();
        const pcm_len = wasmExports.get_sample_buffer_len();
        const pcm = new Float32Array(wasmMemory.buffer, pcm_ptr, pcm_len);
        pcm.set(data, offset);
        this.port.postMessage({ pcmWritten: offset });
      } else if (evaluate && event_input) {
        new Uint8Array(
          wasmMemory.buffer,
          event_input_ptr,
          event_input.length,
        ).set(event_input);
        wasmExports.evaluate();
      } else if (panic) {
        wasmExports.panic();
      }
    };
  }

  process(inputs, outputs, parameters) {
    if (wasmExports && outputs[0][0]) {
      if (input_buffer && inputs[0] && inputs[0][0]) {
        for (let i = 0; i < inputs[0][0].length; i++) {
          const offset = i * CHANNELS;
          for (let c = 0; c < CHANNELS; c++) {
            input_buffer[offset + c] = inputs[0][c]?.[i] ?? inputs[0][0][i];
          }
        }
      }
      wasmExports.dsp();
      const out = outputs[0];
      for (let i = 0; i < out[0].length; i++) {
        const offset = i * CHANNELS;
        for (let c = 0; c < CHANNELS; c++) {
          out[c][i] = output[offset + c];
          if (framebuffer) {
            framebuffer[frameIdx * CHANNELS + c] = output[offset + c];
          }
        }
        frameIdx = (frameIdx + 1) % (framebuffer.length / CHANNELS);
      }

      block++;
      if (block % 8 === 0 && framebuffer) {
        this.port.postMessage({
          framebuffer: framebuffer.slice(),
          frame: frameIdx,
        });
      }

      if (this.clock_active && block % CLOCK_SIZE === 0) {
        this.clockmsg.t0 = this.clockmsg.t1;
        this.clockmsg.t1 = wasmExports.get_time();
        this.port.postMessage(this.clockmsg);
      }
    }
    return this.active;
  }
}
registerProcessor("doux-processor", DouxProcessor);
`;

export class Doux {
	base: string;
	BLOCK_SIZE = BLOCK_SIZE;
	CHANNELS = CHANNELS;
	ready: Promise<void>;
	sampleRate = 0;
	frame: Int32Array = new Int32Array(1);
	framebuffer: Float32Array = new Float32Array(0);
	samplesReady: Promise<void> | null = null;

	private initAudio: Promise<AudioContext>;
	private worklet: AudioWorkletNode | null = null;
	private encoder: TextEncoder | null = null;
	private micSource: MediaStreamAudioSourceNode | null = null;
	private micStream: MediaStream | null = null;
	private onTick?: (msg: ClockMessage) => void;

	constructor(options: DouxOptions = {}) {
		this.base = options.base ?? '/';
		this.onTick = options.onTick;
		this.initAudio = new Promise((resolve) => {
			if (typeof document === 'undefined') return;
			document.addEventListener('click', async function init() {
				const ac = new AudioContext();
				await ac.resume();
				resolve(ac);
				document.removeEventListener('click', init);
			});
		});
		this.ready = this.runWorklet();
	}

	private async initWorklet(): Promise<AudioWorkletNode> {
		const ac = await this.initAudio;
		const blob = new Blob([workletCode], { type: 'application/javascript' });
		const dataURL = URL.createObjectURL(blob);
		await ac.audioWorklet.addModule(dataURL);
		const worklet = new AudioWorkletNode(ac, 'doux-processor', {
			outputChannelCount: [CHANNELS],
			processorOptions: { clock_active: !!this.onTick }
		});
		worklet.connect(ac.destination);
		const res = await fetch(`${this.base}doux.wasm?t=${Date.now()}`);
		const wasm = await res.arrayBuffer();
		return new Promise((resolve) => {
			worklet.port.onmessage = async (e) => {
				if (e.data.ready) {
					this.sampleRate = e.data.sampleRate;
					this.frame = new Int32Array(1);
					this.frame[0] = 0;
					const framebufferLen = Math.floor((this.sampleRate / 60) * CHANNELS) * 4;
					this.framebuffer = new Float32Array(framebufferLen);
					this.samplesReady = douxsamples('https://samples.raphaelforment.fr');
					resolve(worklet);
				} else if (e.data.clock) {
					this.onTick?.(e.data);
				} else if (e.data.framebuffer) {
					this.framebuffer.set(e.data.framebuffer);
					this.frame[0] = e.data.frame;
				}
			};
			worklet.port.postMessage({ wasm });
		});
	}

	private async runWorklet(): Promise<void> {
		const ac = await this.initAudio;
		if (ac.state !== 'running') await ac.resume();
		if (this.worklet) return;
		this.worklet = await this.initWorklet();
	}

	parsePath(path: string): DouxEvent {
		const chunks = path
			.trim()
			.split('\n')
			.map((line) => line.split('//')[0])
			.join('')
			.split('/')
			.filter(Boolean);
		const pairs: [string, string | undefined][] = [];
		for (let i = 0; i < chunks.length; i += 2) {
			pairs.push([chunks[i].trim(), chunks[i + 1]?.trim()]);
		}
		return Object.fromEntries(pairs);
	}

	private encodeEvent(input: string | DouxEvent): Uint8Array {
		if (!this.encoder) this.encoder = new TextEncoder();
		const str =
			typeof input === 'string'
				? input
				: Object.entries(input)
						.map(([k, v]) => `${k}/${v}`)
						.join('/');
		return this.encoder.encode(str + '\0');
	}

	async evaluate(input: DouxEvent): Promise<void> {
		const msg = await this.prepare(input);
		return this.send(msg);
	}

	async hush(): Promise<void> {
		await this.panic();
		const ac = await this.initAudio;
		ac.suspend();
	}

	async resume(): Promise<void> {
		const ac = await this.initAudio;
		if (ac.state !== 'running') await ac.resume();
	}

	async panic(): Promise<void> {
		await this.ready;
		this.worklet?.port.postMessage({ panic: true });
	}

	async enableMic(): Promise<void> {
		await this.ready;
		const ac = await this.initAudio;
		const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
		const source = ac.createMediaStreamSource(stream);
		if (this.worklet) source.connect(this.worklet);
		this.micSource = source;
		this.micStream = stream;
	}

	disableMic(): void {
		if (this.micSource) {
			this.micSource.disconnect();
			this.micSource = null;
		}
		if (this.micStream) {
			this.micStream.getTracks().forEach((t) => t.stop());
			this.micStream = null;
		}
	}

	async prepare(event: DouxEvent): Promise<PreparedMessage> {
		await this.ready;
		if (this.samplesReady) await this.samplesReady;
		await this.maybeLoadFile(event);
		const encoded = this.encodeEvent(event);
		return {
			evaluate: true,
			event_input: encoded
		};
	}

	async send(msg: PreparedMessage): Promise<void> {
		await this.resume();
		this.worklet?.port.postMessage(msg);
	}

	private async fetchSample(url: string): Promise<Float32Array> {
		const ac = await this.initAudio;
		const encoded = encodeURI(url);
		const buffer = await fetch(encoded)
			.then((res) => res.arrayBuffer())
			.then((buf) => ac.decodeAudioData(buf));
		return buffer.getChannelData(0);
	}

	private async loadSound(s: string, n = 0): Promise<SoundInfo> {
		const soundKey = `${s}:${n}`;

		if (loadedSounds.has(soundKey)) {
			return loadedSounds.get(soundKey)!;
		}

		if (!loadingSounds.has(soundKey)) {
			const urls = soundMap.get(s);
			if (!urls) throw new Error(`sound ${s} not found in soundMap`);
			const url = urls[n % urls.length];

			const promise = this.fetchSample(url).then(async (data) => {
				const offset = pcm_offset;
				pcm_offset += data.length;

				await this.sendPcmData(data, offset);

				const info: SoundInfo = {
					pcm_offset: offset,
					frames: data.length,
					channels: 1,
					freq: 65.406
				};
				loadedSounds.set(soundKey, info);
				return info;
			});

			loadingSounds.set(soundKey, promise);
		}

		return loadingSounds.get(soundKey)!;
	}

	private sendPcmData(data: Float32Array, offset: number): Promise<void> {
		return new Promise((resolve) => {
			const handler = (e: MessageEvent) => {
				if (e.data.pcmWritten === offset) {
					this.worklet?.port.removeEventListener('message', handler);
					resolve();
				}
			};
			this.worklet?.port.addEventListener('message', handler);
			this.worklet?.port.postMessage({ writePcm: { data, offset } });
		});
	}

	private async maybeLoadFile(event: DouxEvent): Promise<void> {
		const s = event.s || event.sound;
		if (!s || typeof s !== 'string') return;
		if (sources.includes(s)) return;
		if (!soundMap.has(s)) return;

		const n = typeof event.n === 'string' ? parseInt(event.n) : event.n ?? 0;
		const info = await this.loadSound(s, n);
		event.file_pcm = info.pcm_offset;
		event.file_frames = info.frames;
		event.file_channels = info.channels;
		event.file_freq = info.freq;
	}

	async play(path: string): Promise<void> {
		await this.resume();
		if (this.samplesReady) await this.samplesReady;
		const event = this.parsePath(path);
		await this.maybeLoadFile(event);
		const encoded = this.encodeEvent(event);
		const msg = {
			evaluate: true,
			event_input: encoded
		};
		this.worklet?.port.postMessage(msg);
	}
}

// Singleton instance
export const doux = new Doux();

// Load default samples
douxsamples('github:eddyflux/crate');
