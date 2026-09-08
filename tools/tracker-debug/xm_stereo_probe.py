"""Original channel-identity controls using the existing packed playback pipeline."""
from pathlib import Path
bootstrap = Path(__file__).with_name('xm_offset_probe.py').read_text().split('results=[]')[0]
exec(compile(bootstrap, 'existing-offset-bootstrap', 'exec'))
template = (ROOT/'target/tracker-compatibility/xm-e9-target-standard-up-after-v10/1-fine-128-e5None/probe.xm').read_bytes()
header, packing, rows, size = struct.unpack_from('<IBHH', template, 336)
assert (header, packing, rows, size) == (9, 0, 8, 40)
ins = 336 + header + size
sh = ins + struct.unpack_from('<I', template, ins)[0]
results = []
for bits in (8, 16):
    for stereo in (False, True):
        stage = stage_root/f'{bits}-{int(stereo)}'
        stage.mkdir()
        data = bytearray(template[:sh+40])
        data[345:385] = bytes([49,1,0,0,0]+[0]*35)
        frames = 8363
        amplitude = 32 if bits == 8 else 8192
        def plane(value):
            return (struct.pack('<b', value) if bits == 8 else struct.pack('<h', value)) + bytes((frames-1)*(bits//8))
        payload = plane(amplitude) + (plane(-amplitude) if stereo else b'')
        struct.pack_into('<III', data, sh, len(payload), 0, 0)
        data[sh+12:sh+18] = bytes([32,0,(16 if bits==16 else 0) | (32 if stereo else 0),128,0,0])
        data[sh+18:sh+40] = bytes(22) if args.legacy else b' '*22
        source = stage/'probe.xm'
        source.write_bytes(data+payload)
        capture = c.capture_case(native.capture, ROOT, renderer, stage, source, 1.2, stage/'native.wav')
        c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','0','--output',stage/'reference.wav','--',source])
        item = {'bits':bits,'stereo':stereo,'source_sha256':c.digest(source),'capture':capture}
        for player in ('native','reference'):
            with wave.open(str(stage/(player+'.wav'))) as w:
                pcm = array.array('h',w.readframes(w.getnframes()))
            if sys.byteorder != 'little': pcm.byteswap()
            block = pcm[4410*2:8820*2]
            item[player] = [sum(block[ch::2])/len(block[ch::2]) for ch in (0,1)]
        assert item['reference'][0] > 10 and (item['reference'][1] < -10 if stereo else item['reference'][1] > 10), 'Reference did not recognize authored channel identity'
        item['pass'] = not capture['playing'] and all(a*b>0 for a,b in zip(item['native'],item['reference']))
        results.append(item)
        (stage_root/'report.json').write_text(json.dumps({'renderer':identity,'reference_sha256':c.digest(reference),'scope':'channel identity only, not full stereo acceptance','cases':results},indent=2))
        print(bits,stereo,item['native'],item['reference'],item['pass'],flush=True)
assert len(results)==4 and all(x['pass'] for x in results), 'Channel identity mismatch; retained report.json'
