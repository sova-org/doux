<main class="page">
    <h1>Native</h1>

    <p>
        The web version is fun! However, Doux was ported to Rust in order to be
        used natively in any application that needs an audio engine. You can
        compile Doux on your computer or use it in applications that integrate
        it such as <a href="https://sova.livecoding.fr">Sova</a>
        or <a href="https://cagire.raphaelforment.fr">Cagire</a>. You will need
        the Rust toolchain installed in order to compile it:
        <a href="https://rustup.rs/">Rustup</a>.
    </p>
    <br />
    <section>
        <h2>Binaries</h2>
        <ul>
            <li>
                <code>doux</code>: OSC server, listening on a configurable port
                (defaults to 57120). Use it with other software like
                TidalCycles, Strudel, Cagire or Sova. It will listen to any
                incoming message until you exit the process.
            </li>
            <br />
            <li>
                <code>doux-repl</code>: REPL with readline support and command
                history saved to
                <code>~/.doux_history</code>. Built-in commands:
                <code>.quit</code> (<code>.q</code>),
                <code>.reset</code> (<code>.r</code>),
                <code>.hush</code>,
                <code>.panic</code>,
                <code>.voices</code>,
                <code>.time</code>,
                <code>.stats</code> (<code>.s</code>),
                <code>.help</code> (<code>.h</code>).
            </li>
            <br />
            <li>
                <code>doux-render</code>: offline WAV renderer. Evaluates
                commands and writes the result to a file. Useful for
                non-realtime rendering or batch processing.
            </li>
            <br />
            <li>
                <code>doux-sova</code>: thin-layer used to communicate with
                Sova.
            </li>
        </ul>
    </section>

    <details>
        <summary>Common flags</summary>
        <p>Shared across <code>doux</code> and <code>doux-repl</code>.</p>
        <table>
            <thead>
                <tr>
                    <th>Flag</th>
                    <th>Short</th>
                    <th>Description</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td><code>--samples</code></td>
                    <td><code>-s</code></td>
                    <td>Directory containing audio samples</td>
                </tr>
                <tr>
                    <td><code>--list-devices</code></td>
                    <td></td>
                    <td>List audio devices and exit</td>
                </tr>
                <tr>
                    <td><code>--input</code></td>
                    <td><code>-i</code></td>
                    <td>Input device (name or index)</td>
                </tr>
                <tr>
                    <td><code>--output</code></td>
                    <td><code>-o</code></td>
                    <td>Output device (name or index)</td>
                </tr>
                <tr>
                    <td><code>--channels</code></td>
                    <td></td>
                    <td>Number of output channels (default: 2)</td>
                </tr>
                <tr>
                    <td><code>--buffer-size</code></td>
                    <td><code>-b</code></td>
                    <td>Audio buffer size in samples (default: system)</td>
                </tr>
                <tr>
                    <td><code>--max-voices</code></td>
                    <td></td>
                    <td>Maximum polyphony (default: 32)</td>
                </tr>
                <tr>
                    <td><code>--host</code></td>
                    <td></td>
                    <td>Audio host backend: <code>jack</code>, <code>alsa</code>, or <code>auto</code> (default: auto). On Linux with PipeWire, use <code>jack</code></td>
                </tr>
                <tr>
                    <td><code>--preload</code></td>
                    <td></td>
                    <td>Preload all samples at startup (<code>doux</code> only)</td>
                </tr>
                <tr>
                    <td><code>--diagnose</code></td>
                    <td></td>
                    <td>Run audio diagnostics and exit</td>
                </tr>
                <tr>
                    <td><code>--port</code></td>
                    <td><code>-p</code></td>
                    <td>OSC port (<code>doux</code> only, default: 57120)</td>
                </tr>
            </tbody>
        </table>
    </details>

    <details>
        <summary>Render flags</summary>
        <p>Flags specific to <code>doux-render</code>.</p>
        <table>
            <thead>
                <tr>
                    <th>Flag</th>
                    <th>Short</th>
                    <th>Description</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td><code>--duration</code></td>
                    <td><code>-d</code></td>
                    <td>Duration to render in seconds (required)</td>
                </tr>
                <tr>
                    <td><code>--eval</code></td>
                    <td><code>-e</code></td>
                    <td>Command to evaluate (repeatable)</td>
                </tr>
                <tr>
                    <td><code>--output</code></td>
                    <td><code>-o</code></td>
                    <td>Output WAV file path (required)</td>
                </tr>
                <tr>
                    <td><code>--samples</code></td>
                    <td><code>-s</code></td>
                    <td>Directory containing audio samples</td>
                </tr>
                <tr>
                    <td><code>--sample-rate</code></td>
                    <td></td>
                    <td>Sample rate (default: 48000)</td>
                </tr>
                <tr>
                    <td><code>--channels</code></td>
                    <td></td>
                    <td>Number of output channels (default: 2)</td>
                </tr>
                <tr>
                    <td><code>--max-voices</code></td>
                    <td></td>
                    <td>Maximum polyphony (default: 64)</td>
                </tr>
            </tbody>
        </table>
    </details>

    <details>
        <summary>Multichannel</summary>
        <p>
            The <code>--channels</code> flag enables multichannel output beyond
            stereo. The number of channels is clamped to your device's maximum
            supported count. The <code>orbit</code> parameter controls output routing.
            With N output channels, there are N/2 stereo pairs. Voices on orbit 0
            output to channels 0–1, orbit 1 to channels 2–3, and so on. When the
            orbit exceeds the number of pairs, it wraps around via modulo. Effects
            (delay, reverb) follow the same routing: each orbit's effect bus outputs
            to its corresponding stereo pair.
        </p>
    </details>

    <details>
        <summary>Buffer size</summary>
        <p>
            The <code>--buffer-size</code> flag controls audio latency. Lower values
            mean less latency but require more CPU. Common values: 64, 128, 256,
            512, 1024. At 48kHz, 256 samples gives ~5.3ms latency. If unset, the
            system chooses a default. Use lower values for live performance, higher
            values for stability.
        </p>
    </details>

    <details>
        <summary>Sample loading</summary>
        <p>
            The <code>--samples</code> flag allows you to (lazy-)load audio
            samples to play with. The engine expects a folder containing folders
            of audio samples. Samples will be available using the folder name.
            You can index into a folder by using the <code>/n/</code> command. Check
            the reference to learn more about this. Use <code>--preload</code> to
            load all samples eagerly at startup instead.
        </p>
    </details>
</main>

<style>
    main h1 {
        font-size: 1.4em;
        margin-top: 0;
        margin-bottom: 1.5em;
        color: #000;
    }

    main h2 {
        margin-top: 2em;
        margin-bottom: 0.5em;
        color: #000;
    }

    main section:first-of-type h2 {
        margin-top: 0;
    }

    table {
        width: 100%;
        border-collapse: collapse;
        margin-top: 1em;
    }

    th,
    td {
        text-align: left;
        padding: 8px 12px;
        border: 1px solid #ccc;
    }

    th {
        background: #f5f5f5;
    }

    code {
        background: #f5f5f5;
        padding: 2px 4px;
    }

    details {
        margin-top: 1.5em;
    }

    summary {
        font-size: 1.17em;
        font-weight: bold;
        cursor: pointer;
        color: #000;
        user-select: none;
    }

    details p {
        margin-top: 0.5em;
    }
</style>
