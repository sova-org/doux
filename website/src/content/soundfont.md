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

<CommandEntry name="gm[preset]" type="source">

Plays a General MIDI instrument. The preset name is part of the source: `gmpiano`, `gmstrings`, `gmdrums`, etc. The engine reads envelope and loop data from the soundfont.

<CodeEditor code={`/sound/gmpiano/note/60`} rows={2} />

<CodeEditor code={`/sound/gmstrings/note/48/decay/2`} rows={2} />

<CodeEditor code={`/sound/gmdrums/note/36`} rows={2} />

<CodeEditor code={`/sound/gmflute/note/72`} rows={2} />

<details class="presets">
<summary>Available preset names</summary>
<table><tbody>
<tr><td>0</td><td>gmpiano</td><td>24</td><td>gmguitar</td><td>48</td><td>gmstrings</td><td>72</td><td>gmpiccolo</td></tr>
<tr><td>1</td><td>gmbrightpiano</td><td>25</td><td>gmsteelguitar</td><td>49</td><td>gmslowstrings</td><td>73</td><td>gmflute</td></tr>
<tr><td>4</td><td>gmepiano</td><td>26</td><td>gmjazzguitar</td><td>52</td><td>gmchoir</td><td>74</td><td>gmrecorder</td></tr>
<tr><td>6</td><td>gmharpsichord</td><td>27</td><td>gmcleangt</td><td>56</td><td>gmtrumpet</td><td>75</td><td>gmpanflute</td></tr>
<tr><td>7</td><td>gmclavinet</td><td>29</td><td>gmoverdrive</td><td>57</td><td>gmtrombone</td><td>79</td><td>gmwhistle</td></tr>
<tr><td>8</td><td>gmcelesta</td><td>30</td><td>gmdistgt</td><td>58</td><td>gmtuba</td><td>80</td><td>gmocarina</td></tr>
<tr><td>9</td><td>gmglockenspiel</td><td>33</td><td>gmbass</td><td>60</td><td>gmhorn</td><td>81</td><td>gmlead</td></tr>
<tr><td>10</td><td>gmmusicbox</td><td>34</td><td>gmpickbass</td><td>61</td><td>gmbrass</td><td>82</td><td>gmsawlead</td></tr>
<tr><td>11</td><td>gmvibraphone</td><td>35</td><td>gmfretless</td><td>64</td><td>gmsopranosax</td><td>89</td><td>gmpad</td></tr>
<tr><td>12</td><td>gmmarimba</td><td>36</td><td>gmslapbass</td><td>65</td><td>gmaltosax</td><td>90</td><td>gmwarmpad</td></tr>
<tr><td>13</td><td>gmxylophone</td><td>38</td><td>gmsynthbass</td><td>66</td><td>gmtenorsax</td><td>91</td><td>gmpolysynth</td></tr>
<tr><td>14</td><td>gmbells</td><td>40</td><td>gmviolin</td><td>67</td><td>gmbarisax</td><td>104</td><td>gmsitar</td></tr>
<tr><td>16</td><td>gmorgan</td><td>41</td><td>gmviola</td><td>68</td><td>gmoboe</td><td>105</td><td>gmbanjo</td></tr>
<tr><td>19</td><td>gmchurchorgan</td><td>42</td><td>gmcello</td><td>70</td><td>gmbassoon</td><td>108</td><td>gmkalimba</td></tr>
<tr><td>21</td><td>gmaccordion</td><td>43</td><td>gmcontrabass</td><td>71</td><td>gmclarinet</td><td>114</td><td>gmsteeldrum</td></tr>
<tr><td>22</td><td>gmharmonica</td><td>45</td><td>gmpizzicato</td><td></td><td></td><td></td><td></td></tr>
<tr><td></td><td></td><td>46</td><td>gmharp</td><td></td><td></td><td></td><td></td></tr>
<tr><td></td><td></td><td>47</td><td>gmtimpani</td><td></td><td></td><td></td><td></td></tr>
</tbody></table>
<p>Drums are on a separate bank: use <code>gmdrums</code> or <code>gmpercussion</code>.</p>
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
