all: main

main: main.cpp
	g++ -std=c++11 -Wall -lncurses main.cpp -o main


clean:
	rm main
