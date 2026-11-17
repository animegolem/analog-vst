#!/usr/bin/env python3
"""Reproducible expert triage; source catalog is read-only."""
import collections,csv,json,pathlib,subprocess
D=pathlib.Path(__file__).resolve().parent
ROOT=pathlib.Path('~/.hermes/cache/scratch/avst').expanduser()
source=json.loads((ROOT/'research/analog-vst-research/data/catalog.json').read_text())['candidates']
if (D/'source-audit.csv').exists():
 with (D/'source-audit.csv').open(newline='') as f: checks=list(csv.DictReader(f))
 for c in checks:
  for k in ('n','status','bytes','text_len'):
   if c.get(k): c[k]=int(c[k])
  if not c.get('error'): c.pop('error',None)
else: checks=json.loads((ROOT/'source-checks.json').read_text())
byid={x['id']:x for x in source}; assert len(source)==104 and len(byid)==104
rows={}; tags={}
def add(id,tier,block,note,tag=''):
 assert id in byid and id not in rows,id
 rows[id]={'id':id,'name':byid[id]['name'],'tier':tier,'blocking_feature':block,'notes':note}; tags[id]=set(tag.split('|')) if tag else set()
T1=[
('aion-stratus-ts9','Stock TS-808 values, fixed silicon clip and bass switches; finite op-amp/no rails and BJT storage caveat.'),
('aion-halo-muff','Gilmour 73 value set; fix midrange to stock. Four BJTs, diode pairs; numerical convergence/CPU risk.'),
('northcoast-msk011','One input with remaining inputs in documented unpatched state; one output. Set bias trim and model 2N5088.'),
('musicthing-mini-drive','One input and one output; remove lamp/driver, keep audio stage and loading; Minimoog-derived, not VCS3.'),
('runoffgroove-tonemender','Fixed mid-shift; keep below op-amp saturation. Source has CC BY-NC-SA terms.'),
('musicthing-london-drive','One input and one of phase-split outputs; omit bulb driver, retain splitter loading; select supply jumper.'),
('musicthing-graphic-eq','Seven band controls, one channel, source NE5532 approximated by finite-gain op-amp; avoid clipping; high CPU.'),
('mutable-kinks','Only SIGN full-wave rectifier, one output; omit LOGIC, noise, S&H and indicator; op-amp limits must be checked.'),
('mutable-ears','Only input preamp; external electrical input in place of physical contact mic; omit detector/gate. Avoid saturation.')]
for rank,(id,note) in enumerate(T1,1): add(id,'T1','none for stated reduced circuit',note); rows[id]['rank_if_T1']=rank
T2={
'winterbloom-neptune':('VCA/OTA','One LP/HP route, fixed salt; diode ladder supported but voltage controlled output cell and bias require substantiation.','vca'),
'mutable-ripples':('SSM2164/V2164 VCA cell','One LP output and fixed routing; add realistic gain-control and limiter model; dynamic CV ports are additional.','vca'),
'mutable-blades':('OTA/VCA and nonlinear rail limiting','One filter/wavefolder path with fixed routing; full dual-routing, multiple CVs and outputs additionally need port work.','vca|rail'),
'mutable-veils':('SSM2164/V2164 VCA cell','One channel at fixed CV; full four channels and audio-rate CV additionally need ports.','vca'),
'mutable-blinds':('four-quadrant multiplier','Single multiplier channel with a fixed modulation setting; full carrier/modulator operation needs second input.','multiplier'),
'mutable-shelves':('voltage-controlled gain cell','One band/fixed CV; full EQ has more ports and controls; do not assume fixed gm VCCS implements a VCA.','vca'),
'shruthi-smr4-mkii':('LM13700 OTA/VCA','Fixed cutoff/resonance/gain filter with one audio output; real CV modulation also requires dynamic source ports.','ota'),
'shruthi-four-pole-mission':('SSM2164 and LM13700 VCA/OTA','Fix one pole mix/resonance mode; dynamic switching/CV/other modes need more than this PR.','vca|ota'),
'shruthi-dual-svf':('V2164 VCA cell and zener breakdown','Fix one filter and mode; both models needed; additional port work for dynamic CV.','vca|zener'),
'shruthi-polivoks':('programmable op-amp macromodel','Fix filter and CV bias; needs selected programmable op-amp device characterization and audio/CV ports for full board.','ota'),
'befaco-chopping-kinky':('LM13700 OTA/VCA','One unchopped wavefolder with fixed controls; full chop switching/gate, dual inputs and comparator require more.','ota'),
'northcoast-msk007':('OTA/VCA macromodel','Fixed-CV filter subset; audit BOM/values and clipping devices before implementation.','ota'),
'northcoast-msk009':('OTA macromodel','Fixed CV, one output; inductors themselves are supported, unlike coupled transformers. Verify model and rectifier behavior.','ota'),
'kassutronics-wavefolder':('LM13700 OTA/VCA','One transistor folding path and fixed CV; both pre-fold OTA and overdrive/VCA use the LM13700; review commercial terms.','ota'),
'urei-1176ln':('JFET plus transformer/coupled inductors','Select one revision; JFET gain reduction and input/output transformers both required, plus accurate detector and device parameter fitting.','jfet|transformer'),
'gyraf-gssl':('VCA macromodel plus shared sidechain','Single channel/fixed sidechain is a reduced study; full stereo linked compression requires cross-channel processing and sidechain ports.','vca|multi'),
'gyraf-g9':('triode and coupled transformers','Tube/transformer line path; fixed switches and gain. Both physical models needed; saturation and parameters are nontrivial.','triode|transformer'),
'gyraf-gpultec':('triode and coupled transformers','Fix EQ switches and model passive LC plus makeup amplifier; ordinary inductors already supported.','triode|transformer'),
'aion-solaris-fuzz':('BJT germanium leakage','Bypass pickup simulator (both Q3 JFET and transformer), replace charge pump with DC supply; germanium leakage important to bias.','leakage'),
'aion-helios-rat':('JFET and op-amp rail/slew model','Full audio chain has 2N5485 follower; fix clipping/bass switches; LM308/OP07 saturation and slew need validation.','jfet|rail'),
'aion-aurora-ota-compressor':('LM13700 OTA/VCA','One selected compressor configuration; validate detector time constants, gain reduction and recovery; model OTA bias.','ota'),
'runoffgroove-condor':('JFET Shichman-Hodges','JFET input gain/drive feeds filters; use fixed notch position. Source applies CC BY-NC-SA to published material.','jfet'),
}
for id,(b,n,t) in T2.items(): add(id,'T2',b,n,t)
T3={
'winterbloom-castor-pollux':'DCO reset scheduler, oscillator and instrument host',
'shruthi-lp2-delay':'PT2399 digital delay plus filter control and host',
'musicthing-turing-machine':'clocked sequencer, noise and trigger/CV I/O',
'musicthing-spring-reverb':'spring mechanical/acoustic model and reverb memory',
'erica-diy-bassline':'voice oscillator, envelope, MIDI and host',
'erica-diy-bbd':'clocked BBD delay, modulation and memory',
'erica-diy-swamp':'chaotic/random oscillator and CV/gate routing',
'befaco-kickall':'triggered drum oscillator/envelopes and instrument host',
'befaco-percall':'triggered envelope generators, gate/switch logic and multi-channel mixer',
'befaco-rampage':'function-generator timing, gates and multiple CV paths',
'northcoast-msk013':'two oscillator cores, sync and multi-output timing',
'moog-minimoog-model-d':'oscillators, envelope, MIDI and mono instrument host',
'moog-werkstatt-01':'oscillator, envelope, modulation and instrument host',
'korg-monotron':'oscillator/LFO and instrument host',
'korg-monotron-duo':'coupled oscillators, modulation and instrument host',
'korg-monotron-delay':'oscillator, LFO and digital delay engine',
'korg-monotribe-rhythm':'triggered percussion voice engine and scheduling',
'korg-ms20-vintage':'oscillators, patch matrix, envelopes and instrument host',
'korg-polysix':'polyphony, oscillator/envelopes and BBD ensemble',
'korg-monopoly':'oscillator sync/cross-modulation, allocation and arpeggiation',
'arp-2600':'oscillators, modulation/patch routing, spring and instrument host',
'arp-odyssey':'two oscillators, S&H, duophonic allocation and envelopes',
'oberheim-sem':'oscillators, envelopes and instrument host',
'oberheim-obxa':'polyphonic voice allocation and instrument host',
'sequential-pro-one':'oscillators, sequencer, MIDI/host and modulation',
'sequential-prophet5-rev3':'polyphonic allocation, oscillators and control memory',
'roland-sh101':'oscillator, envelope, sequencing and host',
'roland-tb303':'oscillator, sequencer, accent/slide and host',
'roland-juno106':'DCOs, polyphony and BBD chorus',
'roland-jupiter8':'polyphony, oscillator synchronization and host',
'roland-tr808':'triggered percussion, metallic/noise source and sequencing',
'roland-tr606':'triggered percussion, metallic oscillators and sequencing',
'roland-tr909':'triggered percussion, sample playback and sequencing',
'roland-cr78':'triggered percussion and rhythm sequencing',
'roland-dimension-d':'two modulated BBD delay lines and linked stereo',
'roland-re201':'multihead tape, transport modulation and spring tank',
'teletronix-la2a':'stateful optocell memory, tube and transformer models',
'yamaha-cs80':'polyphonic voices, expression and modulation',
'yamaha-cs15':'oscillators, envelopes and instrument host',
'edp-wasp':'digital divide-down oscillators and instrument host',
'ems-vcs3-synthi-a':'oscillators, patch matrix, envelope and spring',
'roland-system100m':'modular oscillator/envelope/routing infrastructure for whole family',
'korg-ps3100-3300':'divide-down polyphony and per-key envelopes',
'elka-synthex':'polyphony, digitally controlled oscillators and chorus',
'buchla-208':'oscillators, sequencer, spring and optical LPG memory',
'rhodes-tine-piano':'tine/tonebar physical model, hammer and instrument host',
'wurlitzer-200a':'reed/hammer physical model and instrument host',
'hohner-clavinet-d6':'string/hammer physical model, pickups and host',
'hohner-pianet-n':'adhesive pad/reed physical model and host',
'hammond-b3-tonewheel':'tonewheel bank, key routing, scanner and instrument host',
'leslie-122-147':'rotating speaker acoustics, speed memory and room response',
'vox-continental':'oscillator/divider bank, contacts and instrument host',
'farfisa-compact':'oscillator/divider bank, contacts and instrument host',
'aion-blueshift-dc2':'dual modulated BBD delay lines and stereo mix'
}
for id,b in T3.items(): add(id,'T3',b,'Full catalog target; isolated electronic subsystems may fit a lower tier but are not the full target.')
T4={
'acoustic-piano-physical-model':'Use an original waveguide/modal physical-model research project; document any excitation or IR rights.',
'mi-plaits':'Port chosen published DSP engines to an instrument wrapper; audit assets, rates and per-file license.',
'mi-braids':'Port selected published digital engines; GPL vs MIT must be checked per source and embedded resources.',
'mi-rings':'Port licensed modal/string DSP as a resonator effect or instrument, preserving rates and notices.',
'mi-elements':'Port exciter/modal DSP, checking bundled impulse/sample assets and licenses separately.',
'mi-clouds':'Port granular DSP and buffer/memory logic into a plugin effect; retain notices.',
'mi-beads':'Port published DSP independently; audit each source and dependency for MIT scope.',
'mi-warps':'Port published digital cross-modulation code into a two-input effect, not a circuit solver.',
'mi-tides-2018':'Port DSP function generator/oscillator with event and multi-output hosting.',
'mi-edges':'Use its GPL-compatible digital timer/noise implementation with independent host integration.',
'mi-peaks':'Port published envelopes/drum DSP; check embedded code licenses, rates and wrapper.',
'dx7-msfa-dexed':'Port Apache-2.0 MSFA core with notices or use GPLv3 Dexed wrapper and meet its obligations.',
'yamaha-ymfm-family':'Wrap licensed register-level FM chip emulator with scheduler and musical host; check assets.',
'ayumi-ay-ym-psg':'Wrap licensed PSG code with clocking and host interface; retain required notices.',
'sid-libresidfp':'Use GPL-compatible SID emulator with clock/register host; verify license and ROM assets.',
'daisysp-dsp-library':'Use DSP toolkit directly; distinguish MIT DaisySP from separately licensed LGPL components.',
'faust-physical-models':'Use Faust compiler/library with appropriate generated-code and library exception audit.',
'stk-physical-models':'Use STK classes under its exact license; verify any bundled recordings/rawwaves.',
'korg-logue-sdk':'Use the SDK for supported Korg hardware; wrap original DSP separately as VST/CLAP and audit per-file terms.'}
for id,n in T4.items(): add(id,'T4','digital DSP / software platform / generic research',n)
assert set(rows)==set(byid),(sorted(set(byid)-set(rows)),sorted(set(rows)-set(byid)))
for id,r in rows.items():
 x=byid[id]; r['license_status']=x['license_status']
 if x['license_status']=='explicit_open': ln='Asset-specific open grant, not necessarily permissive; inspect exact files and dependencies.'
 elif x['license_status']=='restricted': ln='Restricted/noncommercial or personal terms on source material; do not redistribute drawings; review intended use.'
 else: ln='Public reference, not an open license; manuals/drawings may not be redistributed; transcribe independently.'
 if id.startswith('aion-'): ln='Aion project terms permit commercial projects, subject to PCB resale and obfuscation conditions; no general software or manual redistribution grant established.'
 if id.startswith('runoffgroove-'): ln='Page labels CC BY-NC-SA 3.0; commercial use of covered material requires separate rights; do not redistribute drawings.'
 if id=='northcoast-msk011': ln='Manual/source GPLv3; open does not mean permissive; audit whether copied material creates obligations.'
 if id=='dx7-msfa-dexed': ln='MSFA core Apache-2.0; complete Dexed GPLv3; scope each reused file.'
 r['license_note']=ln
 c=[z for z in checks if z['id']==id]
 if not c: r['source_ok']='not_checked'
 elif any(z.get('status')==200 and z.get('bytes',0)>0 for z in c):
  failures=[z for z in c if 'error' in z]; r['source_ok']='partial' if failures else 'accessible_not_audited'
 else: r['source_ok']='failed_check'
 if id=='northcoast-msk007' and c: r['source_ok']='partial'
 r.setdefault('rank_if_T1','')
 r['notes']+=' Catalog: '+x['architecture']+' Evidence gap: '+x['missing_evidence']
fields=['id','name','tier','blocking_feature','license_status','license_note','source_ok','rank_if_T1','notes']
with (D/'triage.csv').open('w',newline='') as f:
 w=csv.DictWriter(f,fieldnames=fields);w.writeheader();w.writerows(rows[x['id']] for x in source)
with (D/'source-audit.csv').open('w',newline='') as f:
 audit_fields=['id','n','url','final','status','type','bytes','text_len','error']
 w=csv.DictWriter(f,fieldnames=audit_fields,extrasaction='ignore'); w.writeheader(); w.writerows(checks)
count=collections.Counter(r['tier'] for r in rows.values()); print('COUNTS',dict(count))
for tag in ['ota','vca','jfet','transformer','triode','rail','zener','leakage','multi']:
 print('TAG',tag,len([i for i in T2 if tag in tags[i]]),','.join(i for i in T2 if tag in tags[i]))
briefs={
'aion-stratus-ts9':('1','PDF pp. 3–4 parts and p. 6 schematic, v1.0.2','TS-808 fixed silicon clip and stock bass; fix both selector positions; bypass switching omitted','drive, tone, level','~35–45','Single-pole JRC4558 approximation cannot clip at rails or slew; limit range and measure. 500k audio taper differs from original linear; BJT charge storage absent.'),
'aion-halo-muff':('2','PDF pp. 3–4 parts and p. 7 schematic, v2.1.0','Gilmour 73 value set; stock mid-scoop switch position, 4 NPNs and four clipping diodes','sustain, tone, volume','~45–55','No claim for other variants; account for charge storage and CPU; silicon device parameters and input/load.'),
'northcoast-msk011':('3','MSK 011 manual, parts list and complete schematic; choose original NPN version','One audio input; other three connected as unpatched, keep loading and bias, select AC output; omit unneeded DC tap','selected input level, output/bias as applicable','~30–45','Six 2N5088s; preserve internal loads and null-trim; no claim of full 4-input mixer or linked stereo.'),
'musicthing-mini-drive':('4','Mini Drive Rev 1 schematic sheet 1, 15 Oct 2021','Choose high input and one of the parallel outputs; preserve input-jack normalization and loading; omit bulb and its driver','input level','~18–25','Circuit derives from Minimoog external-input feedback, not London/VCS3; lamp driver loads audio through R10: retain equivalent loading if material.'),
'runoffgroove-tonemender':('7','Tonemender schematic image on designer project page','Select HI mid-shift with switch fixed; dual op-amp and passive tone network','bass, mid, treble, level','~20–25','Only non-saturating gain range; original TLC2272 rail-to-rail behavior not supplied; CC BY-NC-SA applies to page.'),
'musicthing-london-drive':('5','London Drive Proto 1 Rev 1 schematic, 14 Nov 2022','Select high input, fixed -12 V jumper and non-inverting output; terminate unselected output and omit bulb driver','high input level (the ganged low input is unpatched)','~20–30','VCS3-inspired. Preserve splitter loading and R10 load where relevant; no external feedback loop or matched two-output claim.'),
'musicthing-graphic-eq':('6','Graphic EQ published schematic PDF','Single audio input/output, seven fixed-frequency active bands, omit indicator LEDs','seven band faders','~65–85','Approximate NE5532 open-loop/GBW, avoid clipping; 9 active op-amp sections may be computationally expensive. Values from graphic, not OCR.'),
'mutable-kinks':('8','Kinks v41 schematic, sheet 1 rectifiers','Full-wave SIGN output J4 only; preserve R5/R8/R9/IC1A and IC1C network and output load; omit LED and other sections','none in original subset','~12–18','Precision rectifier nonlinear feedback may stress convergence and real TL074 slew/rails; restrict signal levels and compare SPICE carefully.'),
'mutable-ears':('9','Ears v40 schematic, sheet 1 input amplifier','External electrical input and amplifier audio output; omit piezo transducer and detector/gate','gain R7','~10–15','OPA1641 is approximated as one-pole; limit to non-clipping range; transducer and detector are not modeled.')}
md=['# Analog-to-VST catalog triage','',f"Scope: 104 distinct IDs at repository commit `f5a9d8d`.[15] Counts: **T1 {count['T1']}, T2 {count['T2']}, T3 {count['T3']}, T4 {count['T4']}**. All rankings are editorial estimates, not model benchmarks.",'','## Interpretation and evidence limits','', 'T1 means a **specifically stated, useful reduced effect** using present primitives; it does not certify all configurations, a completed transcription, all-knob validation or hardware fidelity. Op-amp cases are limited to headroom where no rail/slew behavior is claimed. Select a fixed position for switches, approximate device parameters transparently, retain loading, verify every source value. Kinks SIGN and Ears amplifier are deliberately standalone subsystems, not the full modules. T2 counts include configurations requiring two or more listed extensions; one PR alone does not necessarily unlock a row. T3 is the full catalog target even if an isolated analog circuit could be separately scoped. T4 describes the right route, not an assessment of artistic value.', '', 'Checked both first URLs for 22 priority records using HTTPS and, for PDFs, local extraction/image review; 200 only establishes retrieval, not correct content, page legibility or license. All 44 requests returned HTTP 200; no dead URL was confirmed in this sample. Per-URL results are in `source-audit.csv`. `source_ok` means `accessible_not_audited`, `partial`, `failed_check`, or `not_checked`; it is not a blanket claim on all catalog links. Web-extract was unavailable (backend search-only); HTTPS plus PDF rasterization was used as fallback. MSK-007 PDF exceeded the 20 MB read cap and could not be fully parsed; status 200 is not PDF validation. Three Music Thing schematic PDFs are vector drawings with no extractable text; raster inspection is essential. No newly proposed model was built or validated.', '', 'Licensing: `explicit_open` may include copyleft; `public_reference` is not an open grant; `restricted` describes source/usage terms, not a legal judgment on independent code. Source copies and logos should not be shipped. Aion documents grant commercial project use subject to PCB and obfuscation restrictions, but that is not automatically a blanket grant to redistribute manuals or all software. Obtain qualified legal review for distribution.', '', '## Ranked T1 build briefs','']
for rank,(id,_) in enumerate(T1,1):
 c,doc,config,knobs,parts,caveat=briefs[id]; x=byid[id];md += [f'### {rank}. {x["name"]} (`{id}`)',f'- **Transcribe:** {doc}.[{c}]',f'- **Configuration:** {config}.',f'- **Controls:** {knobs}. **Estimated physical signal-path parts:** {parts}; excludes plugin implementation, power connectors, visual indicators and usually supply bypass. These are rough scoping estimates, not BOM counts.',f'- **Caveats:** {caveat} **Rights:** {rows[id]["license_note"]}','']
md += ['## T2 by primary blocker','']
groups=collections.defaultdict(list)
for id in T2:groups[rows[id]['blocking_feature']].append(id)
for b,ids in groups.items():
 md.append(f'- **{b} ({len(ids)}):** '+ '; '.join(f'{byid[id]["name"]} (`{id}`)' for id in ids)+'.')
md += ['', 'These are disjoint primary-blocker groups, not independent PR benefit counts. Several rows need multiple listed devices; source gaps and full-module port, switch or host work remain.', '', '## T4: use an alternate route','']
for id,n in T4.items():md.append(f'- **{byid[id]["name"]} (`{id}`):** {n}')
md += ['', '## Catalog errors, omissions and caveats found','', '- **Condor** is described as an analog filter network but the designer explicitly identifies a JFET gain/drive stage. It cannot be the complete signal path with present parts; a passive-only filter subset is not the complete effect.[12]', '- **Helios** contains a 2N5485 JFET output follower, and the original LM308 has low slew rate. A generic op-amp plus diode network is not the whole circuit.[11]', '- **Solaris** has a switchable pickup simulator containing both a 2N5457 and a transformer; both are bypassable. The two germanium BJTs need leakage and gain parameters that the present BJT does not model. A generic PNP model is only an explicitly incomplete approximation.[10]', '- **Chopping Kinky** schematic contains LM13700 cells, comparator and analog switch; a generic op-amp/transistor wavefolder does not implement the complete dual/chop module.[16]', '- **Aion Halo** actual current PDF values specify the Gilmour 73 set, while its old legacy version had a continuous Presence option; do not silently mix these. Current Halo uses four BJTs, so it is buildable in the fixed-switch variant without a JFET.[2]', '- **Mini Drive vs London Drive:** Mini is Minimoog-inspired, London explicitly VCS3-derived. Treat them as different input stages; the lamp drivers are not the audio amplifiers.[4][5]', '- **Full-module versus subset fit:** the catalog assessment correctly describes Kinks and Ears as more involved; only the narrow SIGN or audio-preamp subsets qualify as T1 here. Do not label them full module clones.[8][9]', '- **Format licensing:** the local GUIDE says CLAP can stay MIT, but the pinned default wrapper enables GPL VST3 bindings; building a CLAP alongside VST3 does not by itself prove a GPL-free binary. Audit feature flags and link graph, and distinguish the newer MIT Steinberg SDK from the pinned bindings.', '- **Source risks:** missing open grant is not public domain; indexed OCR may be misleading. A 200 OK on a GitHub file page can still be a shell/HTML rather than a readable schematic; confirm the actual diagram and revision. `source_ok` marks availability checks, not verification of all asset URLs.', '- **Repository metadata:** 91 catalog records omit `repository_fit`; CSV was generated from the 104 `candidates` directly without depending on that optional field.','', '## Source list','']
(D/'triage.md').write_text('\n'.join(md)+'\n')
# append ledger-generated sources, not manually keyed
ledger=str(D/'citations.json'); sources=str(pathlib.Path('~/.hermes/skills/research/grounded-citations/scripts/sources.py').expanduser())
subprocess.run(['python3',sources,'--ledger',ledger,'render','--replace-in',str(D/'triage.md')],check=True)
(D/'counts.json').write_text(json.dumps({'tiers':dict(count),'tags':{t:[i for i in T2 if t in tags[i]] for t in ['ota','vca','jfet','transformer','triode','rail','zener','leakage','multi']}},indent=2))
