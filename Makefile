# this file exists for lazy fingers

Q = $(if ${V},,@)

all:
	${Q}cargo build

test:
	${Q}cargo test
