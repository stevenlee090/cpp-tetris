output: main.o screen.o tetris.o
	g++ -std=c++11 -lncurses main.o screen.o tetris.o -o output

# -c flag instruct the compile to generate an object file
main.o: main.cpp
	g++ -std=c++11 -c main.cpp

screen.o: screen.cpp screen.h
	g++ -std=c++11 -c screen.cpp

tetris.o: tetris.cpp tetris.h
	g++ -std=c++11 -c tetris.cpp

clean:
	rm *.o output

# -------------------------------------------------------
# previous makefile stuff

# all: main

# main: main.cpp
# 	g++ -std=c++11 -Wall -lncurses main.cpp -o main

# clean:
# 	rm main
