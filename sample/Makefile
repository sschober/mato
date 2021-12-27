build/default/default.pdf: src/index.tex
	tectonic -X build
src/index.tex: src/index.md
	matote $< > $@
