#include <iostream>
#include <curses.h>
using namespace std;



wstring tetromino[7];

int nFieldWidth = 12;
int nFieldHeight = 18;
unsigned char *pField = nullptr;

int nScreenWidth = 80;      // console screen size x
int nScreenHeight = 30;     // console screen size y

/**
 * given (px, py) and rotation index r
 * return the corresponding tetromino index
 * (0, 90, 180, 270) clockwise rotation for r = (0, 1, 2, 3)
 */
int Rotate(int px, int py, int r)
{
    const int width = 4;
    int index = -1;

    if (r % 4 == 0) {
        // rotation by 0 deg
        index = py * width + px;
    } else if (r % 4 == 1) {
        // rotation by 90 deg
        index = 12 + py - (px * width);
    } else if (r % 4 == 2) {
        // rotation by 180 deg
        index = 15 - (py * width) - px;
    } else if (r % 4 == 3) {
        // rotation by 270 deg
        index = 3 + py + (px * width);
    }

    return 0;
}

int main(int argc, char ** argv)
{
    // create our assets
    tetromino[0].append(L"..X.");
    tetromino[0].append(L"..X.");
    tetromino[0].append(L"..X.");
    tetromino[0].append(L"..X.");

    tetromino[1].append(L"..X.");
    tetromino[1].append(L".XX.");
    tetromino[1].append(L".X..");
    tetromino[1].append(L"....");

    tetromino[2].append(L".X..");
    tetromino[2].append(L".XX.");
    tetromino[2].append(L"..X.");
    tetromino[2].append(L"....");

    tetromino[3].append(L"....");
    tetromino[3].append(L".XX.");
    tetromino[3].append(L".XX.");
    tetromino[3].append(L"....");

    tetromino[4].append(L"..X.");
    tetromino[4].append(L".XX.");
    tetromino[4].append(L"..X.");
    tetromino[4].append(L"....");

    tetromino[5].append(L"....");
    tetromino[5].append(L".XX.");
    tetromino[5].append(L"..X.");
    tetromino[5].append(L"..X.");

    tetromino[6].append(L"....");
    tetromino[6].append(L".XX.");
    tetromino[6].append(L".X..");
    tetromino[6].append(L".X..");

    pField = new unsigned char[nFieldWidth * nFieldHeight];

    for (int x = 0; x < nFieldWidth; x++) {
        for (int y = 0; y < nFieldHeight; y++) {
            // set board to 0 unless it is on the border
            pField[y * nFieldWidth + x] = (x == 0 || x == nFieldWidth - 1 || y == nFieldWidth - 1) ? 9 : 0;
        }
    }

    // init screen
    // set up memory and clear screen
    initscr();

    // y is the number of rows, x is the number of columns
    int x, y;
    x = 10;
    y = 20;

    move(y, x);

    // print a string(const char *) to a window
    printw("Hello World!");

    // refresh screen
    refresh(); 

    // waits for user key input, returns int value of that key
    int c = getch();

    move(0, 0);
    printw("%d", c);

    // clear screen;
    clear();

    getch();

    // deallocate memory and ends ncurses
    endwin();

    return 0;
}