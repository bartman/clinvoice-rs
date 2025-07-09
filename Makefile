Q = $(if ${V},,@)

all:
	${Q}cargo build
