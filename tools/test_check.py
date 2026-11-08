"""Exercise the destructive boundary and incremental behavior of the Windows adapter."""
from pathlib import Path
import tempfile
import unittest
from check import sync_stage


class StageCopyTests(unittest.TestCase):
    def test_updates_and_prunes_only_the_owned_stage(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp)
            src = target / 'stage.new'; src.mkdir()
            dst = target / 'stage/stratos_api'; dst.mkdir(parents=True)
            (src / 'same').write_text('same'); (dst / 'same').write_text('same')
            (src / 'changed').write_text('new'); (dst / 'changed').write_text('old')
            (dst / 'gone').mkdir(); (dst / 'gone/file').write_text('delete')
            neighbor = target / 'stage/other'; neighbor.mkdir(); (neighbor / 'keep').write_text('keep')
            mtime = (dst / 'same').stat().st_mtime_ns
            sync_stage(src, dst, target)
            self.assertEqual((dst / 'changed').read_text(), 'new')
            self.assertEqual((dst / 'same').stat().st_mtime_ns, mtime)
            self.assertFalse((dst / 'gone').exists())
            self.assertEqual((neighbor / 'keep').read_text(), 'keep')

    def test_refuses_other_destinations_before_writing(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp)
            src = target / 'stage.new'; src.mkdir()
            (target / 'keep').write_text('keep')
            with self.assertRaises(ValueError):
                sync_stage(src, target, target)
            self.assertEqual((target / 'keep').read_text(), 'keep')


if __name__ == '__main__':
    unittest.main()
