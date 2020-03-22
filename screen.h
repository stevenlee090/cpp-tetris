#ifndef SCREEN_H
#define SCREEN_H

#include <curses.h>
#include <iostream>

/**
 * Screen class will be responsible to handling all the window/display
 * interaction with ncurses
 * 
 * Note that the screen coordinate system is defined as having (x, y) = (0, 0)
 * on the top left corner of the window/screen.
 * 
 * x coorindate increases as we move to the right of the screen
 * y coordinate increases as we move down the screen
 * 
 */

class Screen {

private:
    int nScreenWidth;
    int nScreenHeight;
    int start_x;
    int start_y;

    WINDOW * w;
    char *screen = nullptr;

    // width and height of the tetris playing field
    int nFieldWidth;
    int nFieldHeight;

    void FillEmptyScreen();

public:
    Screen(int fw, int fh);   // default constructor
    // TODO: optional constructor which can specify tetris field size
    ~Screen();  // destructor
    
    void PrintAndRefreshScreen(int score);
};

#endif // SCREEN_H