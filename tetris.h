#ifndef TETRIS_H
#define TETRIS_H

#include <string>
#include <vector>
#include <iostream>

#define OFFSET 2 // tetris playing field offset from the top and left edge

/**
 * Tetris class will contain all the required tetris logic and components.
 * 
 * This includes tetris piece(s) manipulation and scoring.
 */

class Tetris {

private:
    std::wstring tetromino[7];
    unsigned char *pField = nullptr;

    int nFieldWidth;
    int nFieldHeight;

    void DrawFieldBorders();
    void CreateAssets();

public:
    Tetris(int fw, int fh);   // default constructor
    // Tetris(...);    // can have a constructor which specify board size?
    
    int Rotate(int px, int py, int r);
    bool DoesPieceFit(int nTetromino, int rotation, int pos_x, int pos_y);
    void TestPrint();
};

#endif // TETRIS_H