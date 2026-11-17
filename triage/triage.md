# Analog-to-VST catalog triage

Scope: 104 distinct IDs at repository commit `f5a9d8d`.[15] Counts: **T1 9, T2 22, T3 54, T4 19**. All rankings are editorial estimates, not model benchmarks.

## Interpretation and evidence limits

T1 means a **specifically stated, useful reduced effect** using present primitives; it does not certify all configurations, a completed transcription, all-knob validation or hardware fidelity. Op-amp cases are limited to headroom where no rail/slew behavior is claimed. Select a fixed position for switches, approximate device parameters transparently, retain loading, verify every source value. Kinks SIGN and Ears amplifier are deliberately standalone subsystems, not the full modules. T2 counts include configurations requiring two or more listed extensions; one PR alone does not necessarily unlock a row. T3 is the full catalog target even if an isolated analog circuit could be separately scoped. T4 describes the right route, not an assessment of artistic value.

Checked both first URLs for 22 priority records using HTTPS and, for PDFs, local extraction/image review; 200 only establishes retrieval, not correct content, page legibility or license. All 44 requests returned HTTP 200; no dead URL was confirmed in this sample. Per-URL results are in `source-audit.csv`. `source_ok` means `accessible_not_audited`, `partial`, `failed_check`, or `not_checked`; it is not a blanket claim on all catalog links. Web-extract was unavailable (backend search-only); HTTPS plus PDF rasterization was used as fallback. MSK-007 PDF exceeded the 20 MB read cap and could not be fully parsed; status 200 is not PDF validation. Three Music Thing schematic PDFs are vector drawings with no extractable text; raster inspection is essential. No newly proposed model was built or validated.

Licensing: `explicit_open` may include copyleft; `public_reference` is not an open grant; `restricted` describes source/usage terms, not a legal judgment on independent code. Source copies and logos should not be shipped. Aion documents grant commercial project use subject to PCB and obfuscation restrictions, but that is not automatically a blanket grant to redistribute manuals or all software. Obtain qualified legal review for distribution.

## Ranked T1 build briefs

### 1. Stratus / TS-808–TS-9 overdrive topology (`aion-stratus-ts9`)
- **Transcribe:** PDF pp. 3–4 parts and p. 6 schematic, v1.0.2.[1]
- **Configuration:** TS-808 fixed silicon clip and stock bass; fix both selector positions; bypass switching omitted.
- **Controls:** drive, tone, level. **Estimated physical signal-path parts:** ~35–45; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** Single-pole JRC4558 approximation cannot clip at rails or slew; limit range and measure. 500k audio taper differs from original linear; BJT charge storage absent. **Rights:** Aion project terms permit commercial projects, subject to PCB resale and obfuscation conditions; no general software or manual redistribution grant established.

### 2. Halo / Big Muff transistor-fuzz family (`aion-halo-muff`)
- **Transcribe:** PDF pp. 3–4 parts and p. 7 schematic, v2.1.0.[2]
- **Configuration:** Gilmour 73 value set; stock mid-scoop switch position, 4 NPNs and four clipping diodes.
- **Controls:** sustain, tone, volume. **Estimated physical signal-path parts:** ~45–55; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** No claim for other variants; account for charge storage and CPU; silicon device parameters and input/load. **Rights:** Aion project terms permit commercial projects, subject to PCB resale and obfuscation conditions; no general software or manual redistribution grant established.

### 3. MSK 011 Transistor Mixer (`northcoast-msk011`)
- **Transcribe:** MSK 011 manual, parts list and complete schematic; choose original NPN version.[3]
- **Configuration:** One audio input; other three connected as unpatched, keep loading and bias, select AC output; omit unneeded DC tap.
- **Controls:** selected input level, output/bias as applicable. **Estimated physical signal-path parts:** ~30–45; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** Six 2N5088s; preserve internal loads and null-trim; no claim of full 4-input mixer or linked stereo. **Rights:** Manual/source GPLv3; open does not mean permissive; audit whether copied material creates obligations.

### 4. Mini Drive (`musicthing-mini-drive`)
- **Transcribe:** Mini Drive Rev 1 schematic sheet 1, 15 Oct 2021.[4]
- **Configuration:** Choose high input and one of the parallel outputs; preserve input-jack normalization and loading; omit bulb and its driver.
- **Controls:** input level. **Estimated physical signal-path parts:** ~18–25; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** Circuit derives from Minimoog external-input feedback, not London/VCS3; lamp driver loads audio through R10: retain equivalent loading if material. **Rights:** Public reference, not an open license; manuals/drawings may not be redistributed; transcribe independently.

### 5. Tonemender tone-stack and clean boost (`runoffgroove-tonemender`)
- **Transcribe:** Tonemender schematic image on designer project page.[7]
- **Configuration:** Select HI mid-shift with switch fixed; dual op-amp and passive tone network.
- **Controls:** bass, mid, treble, level. **Estimated physical signal-path parts:** ~20–25; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** Only non-saturating gain range; original TLC2272 rail-to-rail behavior not supplied; CC BY-NC-SA applies to page. **Rights:** Page labels CC BY-NC-SA 3.0; commercial use of covered material requires separate rights; do not redistribute drawings.

### 6. London Drive (`musicthing-london-drive`)
- **Transcribe:** London Drive Proto 1 Rev 1 schematic, 14 Nov 2022.[5]
- **Configuration:** Select high input, fixed -12 V jumper and non-inverting output; terminate unselected output and omit bulb driver.
- **Controls:** high input level (the ganged low input is unpatched). **Estimated physical signal-path parts:** ~20–30; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** VCS3-inspired. Preserve splitter loading and R10 load where relevant; no external feedback loop or matched two-output claim. **Rights:** Public reference, not an open license; manuals/drawings may not be redistributed; transcribe independently.

### 7. Graphic EQ (`musicthing-graphic-eq`)
- **Transcribe:** Graphic EQ published schematic PDF.[6]
- **Configuration:** Single audio input/output, seven fixed-frequency active bands, omit indicator LEDs.
- **Controls:** seven band faders. **Estimated physical signal-path parts:** ~65–85; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** Approximate NE5532 open-loop/GBW, avoid clipping; 9 active op-amp sections may be computationally expensive. Values from graphic, not OCR. **Rights:** Public reference, not an open license; manuals/drawings may not be redistributed; transcribe independently.

### 8. Kinks (`mutable-kinks`)
- **Transcribe:** Kinks v41 schematic, sheet 1 rectifiers.[8]
- **Configuration:** Full-wave SIGN output J4 only; preserve R5/R8/R9/IC1A and IC1C network and output load; omit LED and other sections.
- **Controls:** none in original subset. **Estimated physical signal-path parts:** ~12–18; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** Precision rectifier nonlinear feedback may stress convergence and real TL074 slew/rails; restrict signal levels and compare SPICE carefully. **Rights:** Asset-specific open grant, not necessarily permissive; inspect exact files and dependencies.

### 9. Ears (`mutable-ears`)
- **Transcribe:** Ears v40 schematic, sheet 1 input amplifier.[9]
- **Configuration:** External electrical input and amplifier audio output; omit piezo transducer and detector/gate.
- **Controls:** gain R7. **Estimated physical signal-path parts:** ~10–15; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.
- **Caveats:** OPA1641 is approximated as one-pole; limit to non-clipping range; transducer and detector are not modeled. **Rights:** Asset-specific open grant, not necessarily permissive; inspect exact files and dependencies.

## T2 by primary blocker

- **VCA/OTA (1):** Neptune (`winterbloom-neptune`).
- **SSM2164/V2164 VCA cell (2):** Ripples (2020) (`mutable-ripples`); Veils (2020) (`mutable-veils`).
- **OTA/VCA and nonlinear rail limiting (1):** Blades (`mutable-blades`).
- **four-quadrant multiplier (1):** Blinds (`mutable-blinds`).
- **voltage-controlled gain cell (1):** Shelves (`mutable-shelves`).
- **LM13700 OTA/VCA (4):** Shruthi SMR4 mkII filter board (`shruthi-smr4-mkii`); Chopping Kinky (`befaco-chopping-kinky`); Wavefolder (`kassutronics-wavefolder`); Aurora / OTA guitar compressor family (`aion-aurora-ota-compressor`).
- **SSM2164 and LM13700 VCA/OTA (1):** Shruthi 4-Pole Mission filter board (`shruthi-four-pole-mission`).
- **V2164 VCA cell and zener breakdown (1):** Shruthi Dual SVF filter board (`shruthi-dual-svf`).
- **programmable op-amp macromodel (1):** Shruthi Polivoks filter board (`shruthi-polivoks`).
- **OTA/VCA macromodel (1):** MSK 007 Leapfrog VCF (`northcoast-msk007`).
- **OTA macromodel (1):** MSK 009 Coiler VCF (`northcoast-msk009`).
- **JFET plus transformer/coupled inductors (1):** 1176LN (`urei-1176ln`).
- **VCA macromodel plus shared sidechain (1):** GSSL DIY stereo bus compressor (`gyraf-gssl`).
- **triode and coupled transformers (2):** G9 tube microphone/line preamplifier (`gyraf-g9`); G-Pultec DIY EQ (`gyraf-gpultec`).
- **BJT germanium leakage (1):** Solaris / germanium Fuzz Face topology (`aion-solaris-fuzz`).
- **JFET and op-amp rail/slew model (1):** Helios / RAT distortion topology (`aion-helios-rat`).
- **JFET Shichman-Hodges (1):** Condor analog cabinet simulator (`runoffgroove-condor`).

These are disjoint primary-blocker groups, not independent PR benefit counts. Several rows need multiple listed devices; source gaps and full-module port, switch or host work remain.

## T4: use an alternate route

- **Acoustic piano: generic physical model research (`acoustic-piano-physical-model`):** Use an original waveguide/modal physical-model research project; document any excitation or IR rights.
- **Plaits (`mi-plaits`):** Port chosen published DSP engines to an instrument wrapper; audit assets, rates and per-file license.
- **Braids (`mi-braids`):** Port selected published digital engines; GPL vs MIT must be checked per source and embedded resources.
- **Rings (`mi-rings`):** Port licensed modal/string DSP as a resonator effect or instrument, preserving rates and notices.
- **Elements (`mi-elements`):** Port exciter/modal DSP, checking bundled impulse/sample assets and licenses separately.
- **Clouds (`mi-clouds`):** Port granular DSP and buffer/memory logic into a plugin effect; retain notices.
- **Beads (`mi-beads`):** Port published DSP independently; audit each source and dependency for MIT scope.
- **Warps (`mi-warps`):** Port published digital cross-modulation code into a two-input effect, not a circuit solver.
- **Tides 2018 (`mi-tides-2018`):** Port DSP function generator/oscillator with event and multi-output hosting.
- **Edges (`mi-edges`):** Use its GPL-compatible digital timer/noise implementation with independent host integration.
- **Peaks (`mi-peaks`):** Port published envelopes/drum DSP; check embedded code licenses, rates and wrapper.
- **DX7-style six-operator FM: MSFA and Dexed references (`dx7-msfa-dexed`):** Port Apache-2.0 MSFA core with notices or use GPLv3 Dexed wrapper and meet its obligations.
- **Yamaha OPM/OPN/OPL chip family via ymfm (`yamaha-ymfm-family`):** Wrap licensed register-level FM chip emulator with scheduler and musical host; check assets.
- **AY-3-8910 / YM2149 PSG via Ayumi (`ayumi-ay-ym-psg`):** Wrap licensed PSG code with clocking and host interface; retain required notices.
- **MOS 6581/8580 SID via libresidfp (`sid-libresidfp`):** Use GPL-compatible SID emulator with clock/register host; verify license and ROM assets.
- **DaisySP (supporting DSP toolkit) (`daisysp-dsp-library`):** Use DSP toolkit directly; distinguish MIT DaisySP from separately licensed LGPL components.
- **Faust physical-modeling library (supporting toolkit) (`faust-physical-models`):** Use Faust compiler/library with appropriate generated-code and library exception audit.
- **Synthesis ToolKit in C++ (supporting toolkit) (`stk-physical-models`):** Use STK classes under its exact license; verify any bundled recordings/rawwaves.
- **Korg logue SDK (extension platform) (`korg-logue-sdk`):** Use the SDK for supported Korg hardware; wrap original DSP separately as VST/CLAP and audit per-file terms.

## Catalog errors, omissions and caveats found

- **Condor** is described as an analog filter network but the designer explicitly identifies a JFET gain/drive stage. It cannot be the complete signal path with present parts; a passive-only filter subset is not the complete effect.[12]
- **Helios** contains a 2N5485 JFET output follower, and the original LM308 has low slew rate. A generic op-amp plus diode network is not the whole circuit.[11]
- **Solaris** has a switchable pickup simulator containing both a 2N5457 and a transformer; both are bypassable. The two germanium BJTs need leakage and gain parameters that the present BJT does not model. A generic PNP model is only an explicitly incomplete approximation.[10]
- **Chopping Kinky** schematic contains LM13700 cells, comparator and analog switch; a generic op-amp/transistor wavefolder does not implement the complete dual/chop module.[16]
- **Aion Halo** actual current PDF values specify the Gilmour 73 set, while its old legacy version had a continuous Presence option; do not silently mix these. Current Halo uses four BJTs, so it is buildable in the fixed-switch variant without a JFET.[2]
- **Mini Drive vs London Drive:** Mini is Minimoog-inspired, London explicitly VCS3-derived. Treat them as different input stages; the lamp drivers are not the audio amplifiers.[4][5]
- **Full-module versus subset fit:** the catalog assessment correctly describes Kinks and Ears as more involved; only the narrow SIGN or audio-preamp subsets qualify as T1 here. Do not label them full module clones.[8][9]
- **Format licensing:** the local GUIDE says CLAP can stay MIT, but the pinned default wrapper enables GPL VST3 bindings; building a CLAP alongside VST3 does not by itself prove a GPL-free binary. Audit feature flags and link graph, and distinguish the newer MIT Steinberg SDK from the pinned bindings.
- **Source risks:** missing open grant is not public domain; indexed OCR may be misleading. A 200 OK on a GitHub file page can still be a shell/HTML rather than a readable schematic; confirm the actual diagram and revision. `source_ok` marks availability checks, not verification of all asset URLs.
- **Repository metadata:** 91 catalog records omit `repository_fit`; CSV was generated from the 104 `candidates` directly without depending on that optional field.

## Source list

## Sources

[1] https://aionfx.com/app/files/docs/stratus_documentation.pdf
[2] https://aionfx.com/app/files/docs/halo_documentation.pdf
[3] https://files.northcoastsynthesis.com/msk-011.pdf
[4] https://www.musicthing.co.uk/collateral/mini_drive_Rev1_Schematic.pdf
[5] https://www.musicthing.co.uk/collateral/london_drive_Rev1_Schematic.pdf
[6] https://musicthing.co.uk/collateral/graphic_schematic.pdf
[7] https://runoffgroove.com/tonemender.html
[8] https://pichenettes.github.io/mutable-instruments-documentation/modules/kinks/downloads/kinks_v41.pdf
[9] https://bgr360.github.io/assets/pdf/mutable_instruments/ears_v40.pdf
[10] https://aionfx.com/app/files/docs/solaris_documentation.pdf
[11] https://aionfx.com/app/files/docs/helios_documentation.pdf
[12] https://runoffgroove.com/condor.html
[15] https://github.com/somewheresystems/analog-to-vst/tree/f5a9d8dafdb2691dc8a07fc685e1991b796f3c19
[16] https://befaco.org/docs/Chopping_Kinky/Chopping_Kinky_V1_Schematic.pdf
