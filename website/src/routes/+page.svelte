<script lang="ts">
    import CodeEditor from "$lib/components/CodeEditor.svelte";
</script>

<main class="tutorial">
    <section class="intro">
        <p>
            A monolithic audio engine for live coding ported from C to Rust by <a
                href="https://raphaelforment.fr">BuboBubo</a
            >. The initial project is called
            <a href="https://dough.strudel.cc/">Dough</a>, designed by
            <a href="https://eddyflux.cc/">Felix Roos</a>
            and al. Doux can run in a web browser via
            <a href="https://webassembly.org/">WebAssembly</a>, or natively
            using an an OSC server and/or a REPL. Doux is an opinionated, fixed
            path, semi-modular synth that is remote-controlled via messages. It
            was designed to be used with
            <a href="https://strudel.cc">Strudel</a> and
            <a href="https://tidalcycles.org">TidalCycles</a>.
            <b
                >This fork is a bit special: it adapts and specialize the engine
                for integration with
                <a href="https://sova.livecoding.fr">Sova</a>, a live coding
                environment built in Rust</b
            >.
        </p>
        <p>
            <b>Important note</b>: this project is AGPL 3.0 licensed. We
            encourage you to support the development of the original version
            through the
            <a href="https://opencollective.com/tidalcycles"
                >TidalCycles Open Collective</a
            >. See the license page for more information.
        </p>
        <a href="/support" class="support-link">License & Support</a>
    </section>

    <section>
        <h2>Getting Started</h2>
        <p>
            Click anywhere on the page to start the audio context. Then click
            inside a code block and press <code>Ctrl+Enter</code> to run it, or
            <code>Escape</code> to stop. The easiest way to start is just to
            specify a <code>sound</code> to use:
        </p>
        <CodeEditor code={`/sound/sine`} rows={2} />
        <p>
            You can set the pitch with the <code>/note</code> parameter (MIDI note
            numbers):
        </p>
        <CodeEditor code={`/sound/sine/note/64`} rows={2} />
        <p>
            Or use frequency directly with <code>/freq</code>:
        </p>
        <CodeEditor code={`/sound/sine/freq/330`} rows={2} />
    </section>

    <section>
        <h2>Omitting parameters</h2>
        <p>
            It is possible to omit a large number of parameters or to
            under-specify a voice. Doux has preconfigured defaults for most of
            the core parameters.
        </p>
        <CodeEditor code={`/freq/330`} rows={2} />
        <CodeEditor code={`/spread/5/decay/0.5`} rows={2} />
        <p>
            The default voice is always a <code>tri</code> with sensible envelope
            defaults.
        </p>
    </section>

    <section>
        <h2>Sound Sources</h2>
        <p>
            There are multiple sound sources you can use, detailed in the
            reference. You can also import your own audio samples or use a live
            input as a source.
        </p>
        <CodeEditor code={`/sound/tri`} rows={2} />
        <CodeEditor code={`/sound/saw`} rows={2} />
        <CodeEditor code={`/sound/pulse/pw/0.25`} rows={2} />
        <CodeEditor code={`/sound/white`} rows={2} />
        <CodeEditor code={`/sound/pink`} rows={2} />
        <CodeEditor code={`/sound/analog`} rows={2} />
        <p>Turn on your microphone (and beware of feedback!):</p>
        <CodeEditor code={`/sound/live/gain/2`} rows={2} />
    </section>

    <section>
        <h2>Envelopes</h2>
        <p>
            The amplitude envelope controls how the sound fades in and out. It
            uses the classic ADSR model: attack, decay, sustain, release.
        </p>
        <CodeEditor
            code={`/sound/saw/attack/0.5/decay/0.2/sustain/0.0/release/1`}
            rows={2}
        />
        <p>
            <code>/attack</code> is the time (in seconds) to reach full volume.<br
            />
            <code>/decay</code> is the time to fall to the sustain level.<br />
            <code>/sustain</code> is the level held while the note is on (0-1).<br
            />
            <code>/release</code> is the time to fade to silence after note off.
        </p>
        <p>Try changing each parameter to hear how it affects the sound.</p>
    </section>

    <section>
        <h2>Filters</h2>
        <p>
            Most of the default sources are producing very rich timbres, full of
            harmonics. You are likely to play a lot with filters to remove some
            components of the spectrum. Doux has all the basic filters needed:
        </p>
        <CodeEditor code={`/sound/saw/lpf/800`} rows={2} />
        <p>
            All the basic filters come with a control over resonance <code
                >/..q</code
            >:
        </p>
        <CodeEditor code={`/sound/saw/lpf/800/lpq/10`} rows={2} />
        <CodeEditor code={`/sound/white/hpf/2000`} rows={2} />
        <CodeEditor code={`/sound/white/bpf/1000/bpq/20`} rows={2} />
    </section>

    <section>
        <h2>Effects</h2>
        <p>Doux includes several effects. Here's a sound with reverb:</p>
        <CodeEditor code={`/sound/saw/note/48/reverb/0.8/decay/0.2`} rows={2} />
        <p>And now another sound with a delay:</p>
        <CodeEditor
            code={`/sound/saw/note/60/delay/0.5/delaytime/0.3/delayfeedback/0.6/decay/0.5`}
            rows={2}
        />
        <p>If you stack up effects, it can become quite crazy:</p>
        <CodeEditor
            code={`/sound/saw/note/48/delay/0.5/delaytime/0.1/delayfeedback/0.8/decay/1.5/fanger/0.5/coarse/12/phaser/0.9/gain/1/phaserfeedback/0.9`}
            rows={2}
        />

        <p>
            Note that the order in which the effects is applied is fixed by
            default!
        </p>
    </section>
</main>

<style>
    .tutorial {
        max-width: 650px;
        margin: 0 auto;
        padding: 20px 20px 60px;
        overflow-y: auto;
        height: 100%;
    }

    .tutorial h2 {
        margin-top: 2.5em;
        margin-bottom: 0.5em;
        color: #000;
    }

    .intro {
        padding-bottom: 1em;
    }

    .support-link {
        display: block;
        margin: 1.5em 0;
        padding: 12px 24px;
        background: #f5f5f5;
        border: 1px solid #ccc;
        color: #000;
        text-decoration: none;
        text-align: center;
    }

    .support-link:hover {
        border-color: #999;
        background: #eee;
    }
</style>
