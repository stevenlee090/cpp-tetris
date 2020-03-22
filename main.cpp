// #include <iostream>
// #include <thread>
// #include <chrono>
// #include <vector>
// #include <random>
// #include <ctime>

#include "tetris.h"
#include "screen.h"

#define FW 12
#define FH 18

using namespace std;

int main() {
    
    Screen s(FW, FH);
    Tetris t(FW, FH);

    s.PrintAndRefreshScreen(0);

    // wgetch();

    // t.TestPrint();

    return 0;
}



// int main(int argc, char ** argv)
// {


    // PrintAndRefreshScreen(win, screen);
    // getch();




//     bool game_over = false;

//     int nCurrentPiece = 0;
//     int nCurrentRotation = 0;
//     int nCurrentX = nFieldWidth / 2;
//     int nCurrentY = 0;

//     // bool bKey[4];
//     // bool bRotateHold = false;

//     int nSpeed = 20;
//     int nSpeedCounter = 0;
//     bool bForceDown = false;
//     int nPieceCount = 0; // control game difficulty
//     int nScore = 0;
    
//     srand(time(NULL));
//     vector<int> vLines;

//     while (!game_over)
//     {
//         // game timing -------------------------------
//         this_thread::sleep_for(chrono::milliseconds(50)); // game tick
//         nSpeedCounter++;
//         bForceDown = (nSpeedCounter == nSpeed);

//         // input -------------------------------------
//         int c = wgetch(win);

//         // game logic --------------------------------
//         if (c == KEY_LEFT && DoesPieceFit(nCurrentPiece, nCurrentRotation, nCurrentX - 1, nCurrentY)) {
//             nCurrentX = nCurrentX - 1;
//         }

//         if (c == KEY_RIGHT && DoesPieceFit(nCurrentPiece, nCurrentRotation, nCurrentX + 1, nCurrentY)) {
//             nCurrentX = nCurrentX + 1;
//         }

//         if (c == KEY_DOWN && DoesPieceFit(nCurrentPiece, nCurrentRotation, nCurrentX, nCurrentY + 1)) {
//             nCurrentY = nCurrentY + 1;
//         }

//         if (c == KEY_UP && DoesPieceFit(nCurrentPiece, nCurrentRotation + 1, nCurrentX, nCurrentY)) {
//             nCurrentRotation += 1;
//         }

//         if (bForceDown) {
//             if (DoesPieceFit(nCurrentPiece, nCurrentRotation, nCurrentX, nCurrentY + 1)) {
//                 nCurrentY++;
//             } else {
//                 // --- Lock current piece in the field
//                 for (int px = 0; px < 4; px++) {
//                     for (int py = 0; py < 4; py++) {
//                         if (tetromino[nCurrentPiece][Rotate(px, py, nCurrentRotation)] == L'X') {
//                             pField[(nCurrentY + py) * nFieldWidth + (nCurrentX + px)] = nCurrentPiece + 1;
//                             // +1 in the end because the character string's first character denotes empty space
//                         }
//                     }
//                 }

//                 nPieceCount++;
//                 if (nPieceCount % 10 == 0) {
//                     if (nSpeed >= 10) {
//                         nSpeed--; // decrease game tick
//                     }
//                 }

//                 // --- Check for complete lines 
//                 for (int py = 0; py < 4; py++) {
//                     if (nCurrentY + py < nFieldHeight - 1) {
//                         bool bLine = true;
                        
//                         for (int px = 1; px < nFieldWidth - 1; px++) {
//                             if (pField[(nCurrentY + py) * nFieldWidth + px] == 0) {
//                                 // empty space found
//                                 bLine = false;
//                             } 
//                         }

//                         if (bLine) {
//                             // set line to =
//                             for (int px = 1; px < nFieldWidth - 1; px++) {
//                                 pField[(nCurrentY + py) * nFieldWidth + px] = 8;
//                             }

//                             // store the line number where we have complete line
//                             vLines.push_back(nCurrentY + py);
//                         }
//                     }
//                 }

//                 nScore += 25;

//                 if (!vLines.empty()) {
//                     // left bit shift by vector size
//                     // e.g. if size = 2, then 1 gets bit shifted 2 placed to the left
//                     // resulting in 1 * 2 * 2 = 4
//                     nScore += (1 << vLines.size()) * 100;
//                 }

//                 // --- Choose next piece
//                 nCurrentX = nFieldWidth / 2;
//                 nCurrentY = 0;
//                 nCurrentRotation = 0;
//                 nCurrentPiece = rand() % 7;

//                 // --- If piece does not fit
//                 game_over = !DoesPieceFit(nCurrentPiece, nCurrentRotation, nCurrentX, nCurrentY);
//             }

//             nSpeedCounter = 0;
//         }
        

//         // render output -----------------------------
        
//         // draw the tetris field
//         for (int x = 0; x < nFieldWidth; x++) {
//             for (int y = 0; y < nFieldHeight; y++) {
//                 screen[(y + OFFSET) * nScreenWidth + (x + OFFSET)] = L" ABCDEFG=#"[pField[y * nFieldWidth + x]];
//             }
//         }

//         // draw current piece
//         for (int px = 0; px < 4; px++) {
//             for (int py = 0; py < 4; py++) {
//                 if (tetromino[nCurrentPiece][Rotate(px, py, nCurrentRotation)] == L'X') {
//                     // cout << "Statement is true" << endl;
//                     screen[(nCurrentY + py + OFFSET) * nScreenWidth + (nCurrentX + px + OFFSET)] = nCurrentPiece + 65;
//                 }
//             }
//         }

//         if (!vLines.empty())
//         {
//             PrintAndRefreshScreen(win, screen, nScore);
//             this_thread::sleep_for(chrono::milliseconds(400));

//             for (auto &v : vLines) {
//                 for (int px = 1; px < nFieldWidth - 1; px++) {
//                     // move everything down by one row
//                     for (int py = v; py > 0; py --) {
//                         pField[py * nFieldWidth + px] = pField[(py - 1) * nFieldWidth + px];
//                     }
//                     // set the top row to be 0 (empty)
//                     pField[px] = 0;
//                 }
//             }
            
//             vLines.clear();
//         }

//         // display the frame
//         PrintAndRefreshScreen(win, screen, nScore);
//     }

//     // deallocate memory and ends ncurses
//     endwin();

//     // game over
//     cout << "Game Over! The final score was: " << nScore << endl;

//     return 0;
// }
