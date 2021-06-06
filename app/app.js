import init, { solve } from "./solver/tenxten.js";

export class Board {
    static MOVES = [
        [-3, 0],
        [3, 0],
        [0, -3],
        [0, 3],
        [-2, -2],
        [-2, 2],
        [2, -2],
        [2, 2],
    ];

    constructor(domElm, size) {
        this.domElm = domElm;
        this.size = size;
        this.reset();
        this.disabled = false;
    }

    disable() {
        this.disable = true;
    }

    enable() {
        this.disable = false
    }

    onCellClick(row, column) {
        if (!this.disabled && this.isMovePossible(row, column)) {
            let number = ++this.maxNumber;
            this.setCell(row, column, number);
            this.markPossible();
            if (this.possibleMoves().length === 0) {
                alert("Game Over!");
                this.reset();
            }
        }
    }

    setCell(row, column, number) {
        let cell = this.getCell(row, column);
        cell.innerHTML = number;
        cell.classList.add("placed");
        this.board[row][column] = number;
        this.currentPos = [row, column];
    }

    getCell(row, column) {
        return this.domElm.querySelector(
            `tr:nth-child(${row + 1}) td:nth-child(${column + 1}`
        );
    }

    isEmpty(row, column) {
        return (
            row >= 0 &&
            column >= 0 &&
            row < this.size &&
            column < this.size &&
            this.board[row][column] === 0
        );
    }

    possibleMoves() {
        return Board.MOVES.map(([i, j]) => [
            this.currentPos[0] + i,
            this.currentPos[1] + j,
        ]).filter(([i, j]) => this.isEmpty(i, j));
    }

    isMovePossible(row, column) {
        if (this.currentPos === undefined) {
            return true;
        } else {
            let possible = this.possibleMoves().find(
                ([i, j]) => i == row && j == column
            );
            return this.board[row][column] === 0 && possible !== undefined;
        }
    }

    markPossible() {
        for (let i = 0; i < this.size; i++) {
            for (let j = 0; j < this.size; j++) {
                let cell = this.getCell(i, j);
                if (this.isMovePossible(i, j)) {
                    cell.classList.add("possible");
                } else if (cell.classList.contains("possible")) {
                    cell.classList.remove("possible");
                }
            }
        }
    }

    reset() {
        this.maxNumber = 0;
        this.board = [];
        for (let i = 0; i < this.size; i++) {
            let row = [];
            for (let i = 0; i < this.size; i++) {
                row.push(0);
            }
            this.board.push(row);
        }
        this.currentPos = undefined;
        while (this.domElm.lastChild) {
            this.domElm.removeChild(this.domElm.lastChild);
        }
        for (let i = 0; i < this.size; i++) {
            const row = document.createElement("tr");
            for (let j = 0; j < this.size; j++) {
                let cell = document.createElement("td");
                cell.addEventListener("click", this.onCellClick.bind(this, i, j));
                row.appendChild(cell);
            }
            this.domElm.appendChild(row);
        }
        this.markPossible();
    }

    solve() {
        return new Promise((resolve) => {
            let solution = solve(this.board);
            if (solution === undefined) {
                alert("no solution found!");
                resolve();
            }
            let i = Math.max(1, board.maxNumber);
            let iId = setInterval(() => {
                if (i > this.size * this.size) {
                    clearInterval(iId);
                    resolve();
                    return;
                }
                let [row, column] = this.positionOfNumber(solution, i);
                this.setCell(row, column, i);
                this.markPossible();
                i++;
            }, 20);
        });

    }

    positionOfNumber(board, number) {
        for (let i = 0; i < board.length; i++) {
            for (let j = 0; j < board.length; j++) {
                if (board[i][j] == number)
                    return [i, j]
            }
        }
        return undefined
    }
}

// app starts
await init();

let board = new Board(document.querySelector("#boardTable"), 10);

function showDialog(dialogID) {
    for (let popup of document.querySelectorAll("#overlay .popup")) {
        popup.style.display = "none";
    }
    let overlay = document.getElementById("overlay");
    overlay.style.display = null;
    let elm = document.getElementById(dialogID);
    elm.style.display = "block";
}

function hideDialog() {
    for (let popup of document.querySelectorAll("#overlay .popup")) {
        popup.style.display = "none";
    }
    let overlay = document.getElementById("overlay");
    overlay.style.display = "none";
}

function confirmDialog() {
    return new Promise((resolve) => {
        showDialog("confirm");
        document.getElementById("yes").addEventListener("click", () => {
            hideDialog();
            resolve(true);
        });

        document.getElementById("no").addEventListener("click", () => {
            hideDialog();
            resolve(false);
        });
    });
}

async function restartMaybe() {
    if (await confirmDialog()) {
        board.reset();
        let solveButton = document.getElementById("solveButton");
        solveButton.disabled = false;
    }
}

async function solveBoard() {
    let solveButton = document.getElementById("solveButton");
    let restartButton = document.getElementById("restartButton");
    solveButton.disabled = true;
    restartButton.disabled = true;
    await board.solve();
    restartButton.disabled = false;
}

window.restartMaybe = restartMaybe
window.solveBoard = solveBoard

