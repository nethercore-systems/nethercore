"""Run with: python tools/tracker-debug/test_compatibility.py"""
import struct
import tempfile
import wave
from pathlib import Path
from types import SimpleNamespace
from compatibility import capture_case, compile_compat_renderer


def test_quiet_fixture_gate():
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory) / 'probe.wav'
        for peak in (0, 1, 73, 100, 101, 32767, 32768):
            value = -32768 if peak == 32768 else peak
            with wave.open(str(output), 'wb') as wav:
                wav.setparams((2, 2, 44100, 0, 'NONE', 'not compressed'))
                wav.writeframes(struct.pack('<hh', value, value))

            def rejected_capture(*args):
                raise AssertionError((output, 'silent or clipped', peak))

            try:
                result = capture_case(rejected_capture, None, None, None, None, 1, output)
            except AssertionError:
                assert not 0 < peak <= 100
            else:
                assert 0 < peak <= 100
                assert result['quiet_fixture_peak_pcm16'] == peak

        def unfinished_capture(*args):
            raise AssertionError('Module did not terminate')

        try:
            capture_case(unfinished_capture, None, None, None, None, 1, output)
        except AssertionError as exc:
            assert str(exc) == 'Module did not terminate'
        else:
            raise AssertionError('Termination failure was swallowed')


def test_shared_renderer_adapter():
    with tempfile.TemporaryDirectory() as directory:
        helper = Path(directory) / 'shared.rs'
        original = ('    for id in &tracker.sample_ids {\n'
                    '    let frames = (seconds * 60.0).ceil() as u32;\n')
        helper.write_text(original)
        def compile_probe(root, destination):
            adapted = native.HELPER.read_text()
            assert 'if id.is_empty() { handles.push(0); continue; }' in adapted
            assert 'assert_eq!(first, resumed' in adapted
            raise RuntimeError('compiler failure')
        native = SimpleNamespace(HELPER=helper, compile_renderer=compile_probe)
        try:
            compile_compat_renderer(native, Path(directory) / 'out.exe')
        except RuntimeError as exc:
            assert str(exc) == 'compiler failure'
        else:
            raise AssertionError('compiler failure swallowed')
        assert native.HELPER == helper
        assert helper.read_text() == original
        helper.write_text('changed upstream renderer')
        try:
            compile_compat_renderer(native, Path(directory) / 'out.exe')
        except AssertionError as exc:
            assert 'Shared renderer changed' in str(exc)
        else:
            raise AssertionError('unknown renderer accepted')


if __name__ == '__main__':
    test_quiet_fixture_gate()
    test_shared_renderer_adapter()
    print('Quiet-fixture, failure propagation and shared-renderer adaptation: PASS')
