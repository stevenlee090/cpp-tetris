#ifndef SCREEN_H
#define SCREEN_H

#include <curses.h>

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
    int width;
    int height;
    int start_x;
    int start_y;
    WINDOW * w;

public:
    Screen();   // default constructor
    ~Screen();  // destructor
    
    void PrintAndRefreshScreen(WINDOW *win, char *screen, int score);

};

#endif // SCREEN_H