#include "screen.h"

/**
 * Default constructor for screen
 */
Screen::Screen(int fw, int fh) {

    std::cout << "start of screen constructor" << std::endl;

    // set screen and playing field properties
    nScreenWidth = 80;
    nScreenHeight = 30;
    start_x = 0;
    start_y = 0;

    nFieldWidth = fw;
    nFieldHeight = fh;

    // --- initialise window --- //
    // w = initscr();
    
    initscr();
    w = newwin(nScreenHeight, nScreenWidth, start_y, start_x);

    cbreak();   // let user to quit by ctrl + c
    noecho();   // suppress key press output

    

    keypad(w, true);    // allow arrow keys input to be compared
    
    // DEBUG temporarily disabled for testing
    // nodelay(w, true);   // remove delay for wgetch


    // --- initialise screen --- //
    screen = new char[nScreenWidth * nScreenHeight];
    FillEmptyScreen();

    std::cout << "initialised screen" << std::endl;
}

/**
 * Destructor for Screen class
 */
Screen::~Screen() {
    // deallocate memory and ends ncurses
    std::cout << "Ending ncurses" << std::endl;
    endwin();
}

/**
 * Fill the screen with ' ' space characters
 */
void Screen::FillEmptyScreen() {
    for (int i = 0; i < nScreenWidth * nScreenHeight; i++) {
        screen[i] = ' ';
    }
}

/**
 * Print and refresh a window given a screen input.
 * 
 * The window is provided after it has been initialised.
 * The screen consist of a set number of characters defined by screen width and
 * screen height.
 * 
 * @param win Window which will be refreshed
 * @param screen Screen whose contents will be displayed
 */
void Screen::PrintAndRefreshScreen(int score)
{
    mvwprintw(w, 0, 0, screen);
    mvwprintw(w, 2, nFieldWidth + 6, "SCORE: %8d", score);
    wrefresh(w);

    wgetch(w);

    std::cout << "finished print and refresh" << std::endl;
}