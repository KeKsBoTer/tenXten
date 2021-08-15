
const MOVES = [
    [-3, 0],
    [3, 0],
    [0, -3],
    [0, 3],
    [-2, -2],
    [-2, 2],
    [2, -2],
    [2, 2],
];

const DEFAULT_BOARD_SIZE = 10;

class Board {

    constructor(domElm) {
        this.domElm = domElm;
        this.size = DEFAULT_BOARD_SIZE;
        this.updateListeners = [];
        this.maxNumber = undefined
    }

    isSolved() {
        return this.maxNumber == this.size * this.size
    }

    disable() {
        this.domElm.classList.add("disabled");
        this.disabled = true;
    }

    enable() {
        this.domElm.classList.remove("disabled");
        this.disabled = false;
    }

    onCellClick(row, column) {
        if (!this.disabled && this.isMovePossible(row, column)) {
            this.updateCell(row, column, this.maxNumber + 1);
            this.markPossible();
            if (this.possibleMoves().length === 0)
                if (this.maxNumber != this.size * this.size)
                    showDialog("gameOver");
                else
                    showDialog("winDialog");
        }
    }

    updateCell(row, column, number, noEvent = false) {
        let cell = this.getCell(row, column);
        cell.innerHTML = number || "";
        if (number !== 0)
            cell.classList.add("placed");
        else
            cell.classList.remove("placed");
        this.board[row][column] = number;
        if (number > this.maxNumber) {
            this.maxNumber = number;
            this.currentPos = [row, column];
        }
        if (!noEvent) {
            this.fireUpdateEvent();
        }
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
        return MOVES.map(([i, j]) => [
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
                if (parseInt(cell.innerText) === this.maxNumber) {
                    cell.classList.add("current")
                } else {
                    cell.classList.remove("current")
                }
            }
        }
    }

    reset() {
        this.disabled = false;
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
        this.domElm.innerHTML = ""
        var rows = document.createDocumentFragment();
        for (let i = 0; i < this.size; i++) {
            const row = document.createElement("tr");
            let cellSize = (650 - 5 * this.size) / this.size;
            row.style.height = `${cellSize}px`;
            for (let j = 0; j < this.size; j++) {
                let cell = document.createElement("td");
                cell.addEventListener("click", this.onCellClick.bind(this, i, j));
                cell.style.fontSize = `${cellSize > 30 ? cellSize / 2.2 : cellSize / 3}px`;
                row.appendChild(cell);
            }
            rows.appendChild(row);
        }
        this.domElm.appendChild(rows);
        this.resize();
        this.markPossible();
    }

    async solve() {
        this.worker = this.worker || new Worker("./solver_worker.js");
        let result = await new Promise((resolve, reject) => {
            this.worker.onmessage = ({ data }) => resolve(data);
            this.worker.onmessageerror = reject;
            this.worker.postMessage([board.board])
        });
        return result;
    }

    playSolution(solution) {
        return new Promise((resolve, reject) => {
            let i = Math.max(1, this.maxNumber);
            let iId = setInterval(() => {
                if (i > this.size * this.size) {
                    clearInterval(iId);
                    resolve();
                    return;
                }
                try {
                    let [row, column] = positionOfNumber(solution, i);
                    this.updateCell(row, column, i, true);
                    this.markPossible();
                } catch (e) {
                    clearInterval(iId);
                    reject(e);
                    return;
                } finally {
                    i++;
                }
            }, 20);
        });
    }

    undo() {
        if (this.maxNumber == 0 || this.disabled)
            return
        let [row, column] = this.positionOfNumber(this.maxNumber);
        this.maxNumber--;
        this.updateCell(row, column, 0);
        if (this.maxNumber == 0)
            this.currentPos = undefined
        else
            this.currentPos = this.positionOfNumber(this.maxNumber);
        this.markPossible();
    }

    positionOfNumber(number) {
        return positionOfNumber(this.board, number);
    }

    addCellUpdateListener(callback) {
        this.updateListeners.push(callback)
    }

    fireUpdateEvent() {
        for (let fn of this.updateListeners)
            fn()
    }

    resize() {
        let boardSize = Math.min(window.innerWidth, window.innerHeight - 100);
        let boardWidth = boardSize - (2 * this.size);
        boardTable.style.width = `${boardWidth}px`;
        boardTable.style.height = `${boardWidth}px`;
        buttonToolbar.style.width = `${boardWidth}px`;
    }

    changeSize(size) {
        this.size = size;
        this.reset();
    }
}


function positionOfNumber(board, number) {
    for (let i = 0; i < board.length; i++) {
        for (let j = 0; j < board.length; j++) {
            if (board[i][j] == number)
                return [i, j]
        }
    }
    return undefined
}


let board = new Board(document.querySelector("#boardTable"));

board.addCellUpdateListener(() => {
    if (!board.maxNumber) {
        setButtonsDisabled(true);
    } else if (board.maxNumber == 1) {
        setButtonsDisabled(false);
    } else if (board.maxNumber < board.size * board.size) {
        solveButton.disabled = false
    }
})

function reset() {
    hideDialog();
    board.reset();
    setButtonsDisabled(true);

    boardSizeSelect.innerHTML = undefined;
    for (let i = 5; i <= 20; i++) {
        let opt = document.createElement("option")
        opt.value = i
        opt.innerText = `${i}×${i}`
        opt.selected = i == board.size;
        boardSizeSelect.appendChild(opt)
    }
}

function showDialog(dialogID) {
    for (let popup of document.querySelectorAll("#overlay .popup")) {
        popup.style.display = "none";
    }
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

function alert(title, text) {
    return new Promise((resolve) => {
        showDialog("alertDialog");
        document.querySelector("#alertDialog .title").innerText = title;
        if (text)
            document.querySelector("#alertDialog .text").innerText = text;
        else
            document.querySelector("#alertDialog .text").style.display = "none";
        document.getElementById("cancel").addEventListener("click", () => {
            hideDialog();
            resolve(true);
        });
        document.getElementById("restart").addEventListener("click", () => {
            reset();
            resolve(true);
        });
    });
}

async function restartMaybe() {
    if (await confirmDialog()) {
        reset();
    }
}

async function solveBoard() {
    setButtonsDisabled(true);
    let tId = setTimeout(() => {
        solveButton.classList.add("loading");
    }, 500);
    board.disable();
    try {
        var solution = await board.solve();
    } catch (e) {
        console.error("error while finding solution", e)
    } finally {
        clearTimeout(tId);
        solveButton.classList.remove("loading");
        if (solution)
            await board.playSolution(solution)
        else
            await alert("No solution found", "There is no solution for this board")
        restartButton.disabled = false;
        undoButton.disabled = false;
        board.enable();
    }
}

function setButtonsDisabled(disabled) {
    for (let btn of document.querySelectorAll("#buttonToolbar button:not(#cancelSolve)")) {
        btn.disabled = disabled;
    }
}

async function changeBoardSize(elm) {
    if (board.maxNumber == board.size * board.size || board.maxNumber == 0 || await confirmDialog()) {
        let boardSize = elm.value;
        board.changeSize(boardSize);
        reset();
    } else {
        elm.value = board.size;
    }
}

reset();

window.hideDialog = hideDialog;
window.reset = reset;
window.restartMaybe = restartMaybe;
window.solveBoard = solveBoard;
window.undo = board.undo.bind(board);
window.changeBoardSize = changeBoardSize;
