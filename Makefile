all: main

main: main.cpp
	g++ -std=c++11 -lncurses main.cpp -o main


clean:
	rm main
