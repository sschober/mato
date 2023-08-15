.PHONY: groff

build/default/default.pdf: src/index.tex
	tectonic -X build
src/index.tex: src/index.md
	matote $< > $@
groff:	src/index.md src/_preamble.mom
	matogro src/index.md | cat src/_preamble.mom - | pdfmom -mden > index-groff.pdf