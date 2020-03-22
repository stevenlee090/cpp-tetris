#include "tetris.h"


/**
 * Default constructor for the tetris class
 * 
 * Uses predefined values to initilise the tetris pieces
 */
Tetris::Tetris(int fw, int fh) {
    // create assets and store them in tetromino array
    CreateAssets();

    nFieldWidth = fw;
    nFieldHeight = fh;

    pField = new unsigned char[nFieldWidth * nFieldHeight];
    DrawFieldBorders();


}

/**
 * Draw the left, right and bottom borders of the playing field
 */
void Tetris::DrawFieldBorders() {
    for (int x = 0; x < nFieldWidth; x++)
    {
        for (int y = 0; y < nFieldHeight; y++)
        {
            // set board to 0 unless it is on the border
            pField[y * nFieldWidth + x] = (x == 0 || x == nFieldWidth - 1 || y == nFieldHeight - 1) ? 9 : 0;
        }
    }
}

/**
 * Rotate an arbitrary tetris sub-block and return the resulting block index.
 * 
 * @param px Tetris piece x coordinate
 * @param py Tetris piece y coordinate
 * @return r Resulting tetris piece index
 */
int Tetris::Rotate(int px, int py, int r)
{
    const int width = 4;
    int index = -1;

    if (r % 4 == 0)
    {
        index = py * width + px; // rotate 0 deg
    }
    else if (r % 4 == 1)
    {
        index = 12 + py - (px * width); // rotate 90 deg
    }
    else if (r % 4 == 2)
    {
        index = 15 - (py * width) - px; // rotation by 180 deg
    }
    else if (r % 4 == 3)
    {
        index = 3 - py + (px * width); // rotation by 270 deg
    }

    return index;
}

/**
 * Checks if a tetris piece can fit into a particular board position
 * 
 * @param nTetromino Integer specifying which tetris piece it is
 * @param rotation Integer specifying the orientation of the tetris piece
 * @param pos_x X position of the playing field
 * @param pos_y Y position of the playing field
 */
bool Tetris::DoesPieceFit(int nTetromino, int rotation, int pos_x, int pos_y)
{
    for (int px = 0; px < 4; px++)
    {
        for (int py = 0; py < 4; py++)
        {
            // get index into piece
            int pi = Rotate(px, py, rotation);

            // get index into field
            int fi = (pos_y + py) * nFieldWidth + (pos_x + px);

            if (pos_x + px >= 0 && pos_x + px < nFieldWidth)
            {
                if (pos_y + py >= 0 && pos_y + py < nFieldHeight)
                {
                    if (tetromino[nTetromino][pi] == L'X' && pField[fi] != 0)
                    {
                        return false; // collision detected
                    }
                }
            }
        }
    }
    return true;
}

void Tetris::CreateAssets() {
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
}

void Tetris::TestPrint() {
    std::cout << "test message" << std::endl;
}