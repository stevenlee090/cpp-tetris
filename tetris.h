#ifndef TETRIS_H
#define TETRIS_H

#include <string>

/**
 * Tetris class will contain all the required tetris logic and components.
 * 
 * This includes tetris piece(s) manipulation and scoring.
 */

class Tetris {
    wstring tetromino[7];

public:
    Tetris();   // default constructor
    // Tetris(...);    // can have a constructor which specify board size?
    
    int Rotate(int px, int py, int r);
    bool DoesPieceFit(int nTetromino, int rotation, int pos_x, int pos_y);
    
};

#endif // TETRIS_H