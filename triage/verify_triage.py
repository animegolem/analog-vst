import collections,csv,json,pathlib,subprocess
D=pathlib.Path(__file__).resolve().parent
A=pathlib.Path('~/.hermes/cache/scratch/avst/research/analog-vst-research/data/catalog.json').expanduser()
catalog=json.loads(A.read_text())['candidates']
with (D/'triage.csv').open(newline='') as f: rows=list(csv.DictReader(f))
with (D/'source-audit.csv').open(newline='') as f: audit=list(csv.DictReader(f))
assert len(rows)==len(catalog)==104
assert len(set(r['id'] for r in rows))==104
assert {r['id'] for r in rows}=={x['id'] for x in catalog}
assert all(r['tier'] in ('T1','T2','T3','T4') for r in rows)
assert all(r['license_status'] in ('explicit_open','public_reference','restricted') for r in rows)
assert all(r['blocking_feature'] and r['license_note'] and r['notes'] for r in rows)
top=sorted([r for r in rows if r['tier']=='T1'],key=lambda r:int(r['rank_if_T1']))
assert [int(x['rank_if_T1']) for x in top]==list(range(1,len(top)+1))
assert all(not r['rank_if_T1'] for r in rows if r['tier']!='T1')
assert len(audit)==44 and len({r['id'] for r in audit})==22
assert all(r['status']=='200' for r in audit)
assert next(r for r in rows if r['id']=='northcoast-msk007')['source_ok']=='partial'
counts=collections.Counter(r['tier'] for r in rows)
assert counts=={'T1':9,'T2':22,'T3':54,'T4':19}
tags=json.loads((D/'counts.json').read_text())['tags']
assert len(set(tags['ota'])|set(tags['vca']))==15
for p in ['triage.md','pr-opportunities.md']:
 assert (D/p).exists() and (D/p).stat().st_size>1000
print('Verified:',len(rows),'unique IDs; tiers',dict(counts),'source URLs',len(audit),'ranks',[r['id'] for r in top])
print('T2 OTA/VCA union:',len(set(tags['ota'])|set(tags['vca'])))
print('Source status counts:',dict(collections.Counter(r['source_ok'] for r in rows)))
