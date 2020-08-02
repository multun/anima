all: shader.frag.spv shader.vert.spv

%.spv: %
	glslc -o $@ $<
