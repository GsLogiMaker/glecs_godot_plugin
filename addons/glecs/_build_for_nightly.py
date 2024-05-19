
import os
import regex

os.remove("glecs.gdextension")
os.rename("glecs.gdextension.release", "glecs.gdextension")
os.rename(".gitignore.release", ".gitignore")

name_ptrn = regex.compile(r"""name\s*=\s*\"(.*)\"""")
version_ptrn = regex.compile(r"""version\s*=\s*\"(.*)\"""")

with open("plugin.cfg", "r+") as f:
	txt = f.read()
	
	start, end = name_ptrn.search(txt).span(1)
	txt = txt[:start] + "GlecsNightly" + txt[end:]

	start, end = version_ptrn.search(txt).span(1)
	txt = txt[:end] + "-nightly" + txt[end:]
	
	f.seek(0)
	f.write(txt)
	f.truncate()

exit(0)