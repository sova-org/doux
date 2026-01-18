import { doux } from './doux';

let ctx: CanvasRenderingContext2D | null = null;
let raf: number | null = null;

const lerp = (v: number, min: number, max: number) => v * (max - min) + min;
const invLerp = (v: number, min: number, max: number) => (v - min) / (max - min);
const remap = (v: number, vmin: number, vmax: number, omin: number, omax: number) =>
	lerp(invLerp(v, vmin, vmax), omin, omax);

function drawBuffer(
	ctx: CanvasRenderingContext2D,
	samples: Float32Array,
	channels: number,
	channel: number,
	ampMin: number,
	ampMax: number
) {
	const lineWidth = 2;
	ctx.lineWidth = lineWidth;
	ctx.strokeStyle = 'black';
	const perChannel = samples.length / channels / 2;
	const pingbuffer = doux.frame[0] > samples.length / 2;
	const s0 = pingbuffer ? 0 : perChannel;
	const s1 = pingbuffer ? perChannel : perChannel * 2;
	const px0 = ctx.lineWidth;
	const px1 = ctx.canvas.width - ctx.lineWidth;
	const py0 = ctx.lineWidth;
	const py1 = ctx.canvas.height - ctx.lineWidth;
	ctx.beginPath();
	for (let px = 1; px <= ctx.canvas.width; px++) {
		const si = remap(px, px0, px1, s0, s1);
		const idx = Math.floor(si) * channels + channel;
		const amp = samples[idx];
		if (amp >= 1) ctx.strokeStyle = 'red';
		const py = remap(amp, ampMin, ampMax, py1, py0);
		px === 1 ? ctx.moveTo(px, py) : ctx.lineTo(px, py);
	}
	ctx.stroke();
}

function drawScope() {
	if (!ctx) return;
	ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
	for (let c = 0; c < doux.CHANNELS; c++) {
		drawBuffer(ctx, doux.framebuffer, doux.CHANNELS, c, -1, 1);
	}
	raf = requestAnimationFrame(drawScope);
}

export function initScope(canvas: HTMLCanvasElement) {
	function resize() {
		canvas.width = canvas.clientWidth * devicePixelRatio;
		canvas.height = canvas.clientHeight * devicePixelRatio;
	}
	resize();
	ctx = canvas.getContext('2d');
	const observer = new ResizeObserver(resize);
	observer.observe(canvas);
	return () => observer.disconnect();
}

export function startScope() {
	if (!raf && ctx) {
		drawScope();
	}
}

export function stopScope() {
	if (raf) {
		cancelAnimationFrame(raf);
		raf = null;
	}
	if (ctx) {
		ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
	}
}

let activeResetCallback: (() => void) | null = null;

export function registerActiveEditor(resetCallback: () => void) {
	if (activeResetCallback && activeResetCallback !== resetCallback) {
		activeResetCallback();
	}
	activeResetCallback = resetCallback;
}

export function unregisterActiveEditor(resetCallback: () => void) {
	if (activeResetCallback === resetCallback) {
		activeResetCallback = null;
	}
}

export function resetActiveEditor() {
	if (activeResetCallback) {
		activeResetCallback();
		activeResetCallback = null;
	}
}
