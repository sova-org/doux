---
title: "Soundfont"
slug: "soundfont"
group: "sources"
order: 3
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

General MIDI playback using SF2 soundfont files. Native engine only — examples on this page won't produce sound in the browser. Place an `.sf2` file inside your samples directory and start doux with `--samples`.

<CommandEntry name="gm" type="source">

Plays a General MIDI instrument. Use `/n/` to select a program by name (e.g. `piano`, `strings`, `drums`) or by number (0–127). The engine reads envelope and loop data from the soundfont.

<CodeEditor code={`/sound/gm/n/piano/note/60`} rows={2} />

<CodeEditor code={`/sound/gm/n/strings/note/48/decay/2`} rows={2} />

<CodeEditor code={`/sound/gm/n/drums/note/36`} rows={2} />

<CodeEditor code={`/sound/gm/n/0/note/72`} rows={2} />

<details class="presets">
<summary>Available preset names</summary>
<table><tbody>
<tr><td>0</td><td>piano</td><td>24</td><td>guitar</td><td>48</td><td>strings</td><td>72</td><td>piccolo</td></tr>
<tr><td>1</td><td>brightpiano</td><td>25</td><td>steelguitar</td><td>49</td><td>slowstrings</td><td>73</td><td>flute</td></tr>
<tr><td>4</td><td>epiano</td><td>26</td><td>jazzguitar</td><td>52</td><td>choir</td><td>74</td><td>recorder</td></tr>
<tr><td>6</td><td>harpsichord</td><td>27</td><td>cleangt</td><td>56</td><td>trumpet</td><td>75</td><td>panflute</td></tr>
<tr><td>7</td><td>clavinet</td><td>29</td><td>overdrive</td><td>57</td><td>trombone</td><td>79</td><td>whistle</td></tr>
<tr><td>8</td><td>celesta</td><td>30</td><td>distgt</td><td>58</td><td>tuba</td><td>80</td><td>ocarina</td></tr>
<tr><td>9</td><td>glockenspiel</td><td>33</td><td>bass</td><td>60</td><td>horn</td><td>81</td><td>lead</td></tr>
<tr><td>10</td><td>musicbox</td><td>34</td><td>pickbass</td><td>61</td><td>brass</td><td>82</td><td>sawlead</td></tr>
<tr><td>11</td><td>vibraphone</td><td>35</td><td>fretless</td><td>64</td><td>sopranosax</td><td>89</td><td>pad</td></tr>
<tr><td>12</td><td>marimba</td><td>36</td><td>slapbass</td><td>65</td><td>altosax</td><td>90</td><td>warmpad</td></tr>
<tr><td>13</td><td>xylophone</td><td>38</td><td>synthbass</td><td>66</td><td>tenorsax</td><td>91</td><td>polysynth</td></tr>
<tr><td>14</td><td>bells</td><td>40</td><td>violin</td><td>67</td><td>barisax</td><td>104</td><td>sitar</td></tr>
<tr><td>16</td><td>organ</td><td>41</td><td>viola</td><td>68</td><td>oboe</td><td>105</td><td>banjo</td></tr>
<tr><td>19</td><td>churchorgan</td><td>42</td><td>cello</td><td>70</td><td>bassoon</td><td>108</td><td>kalimba</td></tr>
<tr><td>21</td><td>accordion</td><td>43</td><td>contrabass</td><td>71</td><td>clarinet</td><td>114</td><td>steeldrum</td></tr>
<tr><td>22</td><td>harmonica</td><td>45</td><td>pizzicato</td><td></td><td></td><td></td><td></td></tr>
<tr><td></td><td></td><td>46</td><td>harp</td><td></td><td></td><td></td><td></td></tr>
<tr><td></td><td></td><td>47</td><td>timpani</td><td></td><td></td><td></td><td></td></tr>
</tbody></table>
<p>Drums are on a separate bank: use <code>drums</code> or <code>percussion</code>.</p>
</details>

</CommandEntry>

<style>
  .presets summary {
    font-size: 0.85em;
    color: #999;
    cursor: pointer;
    padding: 4px 0;
  }
  .presets table {
    font-size: 0.8em;
    color: #666;
    border-collapse: collapse;
    width: 100%;
    margin: 8px 0 4px;
  }
  .presets td {
    padding: 1px 8px 1px 0;
    white-space: nowrap;
  }
  .presets td:nth-child(odd) {
    color: #999;
    font-variant-numeric: tabular-nums;
    text-align: right;
    width: 2em;
  }
  .presets p {
    font-size: 0.8em;
    color: #999;
    margin: 6px 0 0;
  }
</style>
