<script lang="ts">
    import CodeEditor from "$lib/components/CodeEditor.svelte";
</script>

<main class="page">
    <section class="intro">
        <p>
            A monolithic audio engine for live coding ported from C to Rust and
            then extended by <a href="https://raphaelforment.fr">BuboBubo</a>.
            The initial project is called
            <a href="https://dough.strudel.cc/">Dough</a> and was designed by
            <a href="https://eddyflux.cc/">Felix Roos</a>
            and al. Doux can run in a web browser via
            <a href="https://webassembly.org/">WebAssembly</a>, or natively
            using an an OSC server and/or a REPL. Doux is an opinionated, fixed
            path, semi-modular synth that is remote-controlled via messages. It
            has been designed initially to be used with
            <a href="https://strudel.cc">Strudel</a>. This fork is a bit
            special: it adapts and specialize the engine for integration with
            <a href="https://sova.livecoding.fr">Sova</a> and
            <a href="https://cagire.raphaelforment.fr">Cagire</a>, two live
            coding environments built with Rust.
        </p>
        <p>
            This project is AGPL 3.0 licensed. We encourage you to support the
            development of the original version through the
            <a href="https://opencollective.com/tidalcycles"
                >TidalCycles Open Collective</a
            >. Consult the support page for more information.
        </p>
        <a href="/support" class="support-link">License & Support</a>
    </section>

    <section>
        <h2>Getting Started</h2>
        <p>
            Click anywhere on the page to start the audio context. Then click
            inside a code block and press <code>Ctrl+Enter</code> to run it, or
            <code>Escape</code> to stop. You can also press the play button if
            you don't want to edit and just want to preview. The easiest way to
            start making sound with Doux is just to specify a <code>sound</code>
            to use:
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
            It is possible to omit pretty much all parameters or to
            under-specify the parameters a synthesis voice you wish to play.
            Doux has preconfigured defaults for most of the core parameters.
        </p>
        <CodeEditor code={`/freq/330`} rows={2} />
        <CodeEditor code={`/spread/5/decay/0.5`} rows={2} />
        <p>
            If no sound source is specified, the voice defaults to
            <code>tri</code> with sensible envelope defaults. If a sound name is
            not recognized, the voice is silently skipped.
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
            Most of the default sources are producing rich timbres full of
            harmonics. Use filters to remove some components of the spectrum.
            Doux has all the basic filters you need:
        </p>
        <CodeEditor code={`/sound/saw/lpf/800`} rows={2} />
        <p>
            All filters come with a control over resonance <code>/..q</code>:
        </p>
        <CodeEditor code={`/sound/saw/lpf/800/lpq/10`} rows={2} />
        <CodeEditor code={`/sound/white/hpf/2000`} rows={2} />
        <CodeEditor code={`/sound/white/bpf/1000/bpq/20`} rows={2} />
    </section>

    <section>
        <h2>Effects</h2>
        <p>Doux includes several effects. Here is a sound with reverb:</p>
        <CodeEditor code={`/sound/saw/note/48/reverb/0.8/decay/0.2`} rows={2} />
        <p>And another sound with delay:</p>
        <CodeEditor
            code={`/sound/saw/note/60/delay/0.5/delaytime/0.3/delayfeedback/0.6/decay/0.5`}
            rows={2}
        />
        <p>If you stack up effects, it can become quite crazy:</p>
        <CodeEditor
            code={`/sound/saw/note/48/delay/0.5/delaytime/0.1/delayfeedback/0.8/decay/1.5/fanger/0.5/coarse/12/phaser/0.9/gain/1/phaserfeedback/0.9/width/2`}
            rows={2}
        />

        <p>
            Note that the order in which the effects are applied is fixed by
            default! You cannot re-order the synthesis chain.
        </p>
    </section>

    <section>
        <h2>Inline Modulation</h2>
        <p>
            Many parameters support <strong>inline modulation</strong>: instead
            of a static value, you write an expression with an operator that
            describes the <em>motion</em> between two values.
            In the reference, parameters marked with
            <code>~</code> support this syntax.
        </p>

        <h3>Oscillate <code>~</code></h3>
        <p>Cycle between two values forever. Syntax: <code>min~max:period[shape]</code></p>
        <ul>
            <li><code>200~4000:2</code> — sine (default)</li>
            <li><code>200~4000:2t</code> — triangle</li>
            <li><code>200~4000:2w</code> — saw (ramp up, snap down)</li>
            <li><code>200~4000:2q</code> — square (alternate min/max)</li>
        </ul>

        <p>A lowpass filter sweeping from 200Hz to 4000Hz over 2 seconds:</p>
        <CodeEditor code={`/sound/saw/lpf/200~4000:2`} rows={2} />

        <p>Pan bouncing left to right with a triangle wave:</p>
        <CodeEditor code={`/sound/saw/pan/0~1:1t`} rows={2} />

        <h3>Transition <code>&gt;</code></h3>
        <p>Go from A to B, hold at B. Chain segments for multi-point envelopes.
           Syntax: <code>start&gt;target:duration[curve]</code></p>
        <ul>
            <li><code>200&gt;4000:2</code> — linear (default)</li>
            <li><code>200&gt;4000:2e</code> — exponential</li>
            <li><code>200&gt;4000:2s</code> — smooth (ease in-out)</li>
        </ul>

        <p>Multi-segment chains:</p>
        <ul>
            <li><code>200&gt;4000:1&gt;800:2</code> — up in 1s, down in 2s</li>
            <li><code>0&gt;1:0.01&gt;0:2</code> — percussive blip</li>
            <li><code>200&gt;4000:1e&gt;200:1.5s</code> — exponential up, smooth down</li>
        </ul>

        <p>Append <code>~</code> to the last duration to loop:</p>
        <ul>
            <li><code>200&gt;4000:1&gt;200:1~</code> — looping triangle-like sweep</li>
        </ul>

        <p>FM index ramping up exponentially over 3 seconds:</p>
        <CodeEditor code={`/fm/0>8:3e/fmh/3/decay/3`} rows={2} />

        <h3>Random <code>?</code></h3>
        <p>Wander stochastically within a range. Syntax: <code>min?max:period[shape]</code></p>
        <ul>
            <li><code>200?4000:0.5</code> — sample-and-hold (default)</li>
            <li><code>200?4000:0.5s</code> — smooth (cosine-interpolated)</li>
            <li><code>200?4000:0.1d</code> — drunk walk (brownian)</li>
        </ul>

        <p>Random pan jumps:</p>
        <CodeEditor code={`/sound/saw/pan/0?1:0.25`} rows={2} />

    </section>
</main>

<style>
    main h2 {
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
