import sys, zipfile, re
from xml.etree import ElementTree as ET
path = sys.argv[1]
z = zipfile.ZipFile(path)
wb = z.read('xl/workbook.xml').decode('utf-8')
print("sheets:", re.findall(r'name="([^"]+)"', wb))
# shared strings first 40
try:
    ss = ET.fromstring(z.read('xl/sharedStrings.xml'))
    strs = [''.join(t.text or '' for t in si.iter('{http://schemas.openxmlformats.org/spreadsheetml/2006/main}t')) for si in ss]
    print("first strings:", strs[:40])
except KeyError:
    print("no sharedStrings")
