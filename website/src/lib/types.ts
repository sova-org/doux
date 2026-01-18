export interface DouxEvent {
	doux?: string;
	s?: string;
	sound?: string;
	n?: string | number;
	freq?: number;
	wave?: string;
	file_pcm?: number;
	file_frames?: number;
	file_channels?: number;
	file_freq?: number;
	[key: string]: string | number | undefined;
}

export interface SoundInfo {
	pcm_offset: number;
	frames: number;
	channels: number;
	freq: number;
}

export interface ClockMessage {
	clock: boolean;
	t0: number;
	t1: number;
	latency: number;
}

export interface DouxOptions {
	onTick?: (msg: ClockMessage) => void;
	base?: string;
}

export interface PreparedMessage {
	evaluate: boolean;
	event_input: Uint8Array;
}
