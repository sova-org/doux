<main class="native-page">
    <h1>Native</h1>

    <p>
        The web version is really fun if you want to build a website that uses a
        robust open source audio engine. However, Doux was ported from C mostly
        to be used natively. You will need to compile it yourself or use it
        directly as it is integrated in <a href="https://sova.livecoding.fr"
            >Sova</a
        >. Read the instructions in the repo to learn how to compile it. You
        will need the Rust toolchain that you can get using
        <a href="https://rustup.rs/">Rustup</a>.
    </p>
    <br />
    <section>
        <h2>Binaries</h2>
        <ul>
            <li>
                <code>doux</code> runs as an OSC server, listening on a configurable
                port (default 57120). Use it with TidalCycles, Strudel, or Sova.
                It will listen to any incoming message until it is killed.
            </li>
            <br />
            <li>
                REPL with readline support and command history saved to
                <code>~/.doux_history</code>. Built-in commands:
                <code>.quit</code>, <code>.reset</code>, <code>.hush</code>,
                <code>.panic</code>, <code>.voices</code>, <code>.time</code>.
            </li>
        </ul>
    </section>

    <section>
        <h2>Flags</h2>
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
                    <td><code>--port</code></td>
                    <td><code>-p</code></td>
                    <td>OSC port, <code>doux</code> only (default: 57120)</td>
                </tr>
            </tbody>
        </table>
    </section>

    <section>
        <h2>Multichannel</h2>
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
    </section>

    <section>
        <h2>Buffer size</h2>
        <p>
            The <code>--buffer-size</code> flag controls audio latency. Lower values
            mean less latency but require more CPU. Common values: 64, 128, 256, 512,
            1024. At 48kHz, 256 samples gives ~5.3ms latency. If unset, the system
            chooses a default. Use lower values for live performance, higher values
            for stability.
        </p>
    </section>

    <section>
        <h2>Sample loading</h2>
        <p>
            The <code>--samples</code> flag allows you to (lazy-)load audio
            samples to play with. The engine expects a folder containing folders
            of audio samples. Samples will be available using the folder name.
            You can index into a folder by using the <code>/n/</code> command. Check
            the reference to learn more about this.
        </p>
    </section>
</main>

<style>
    .native-page {
        max-width: 650px;
        margin: 0 auto;
        padding: 20px 20px 60px;
        overflow-y: auto;
        height: 100%;
    }

    .native-page h1 {
        font-size: 18px;
        margin-top: 0;
        margin-bottom: 1.5em;
        color: #000;
    }

    .native-page h2 {
        margin-top: 2em;
        margin-bottom: 0.5em;
        color: #000;
    }

    .native-page section:first-of-type h2 {
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
</style>
