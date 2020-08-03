all: shader.frag.spv shader.vert.spv fxaa.frag.spv fxaa.vert.spv

%.spv: %
	glslc -o $@ $<
