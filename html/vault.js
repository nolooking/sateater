const colorList = ["black", "red", "cyan", "green", "yellow"];

function get_board() {
    fetch(
        "/api/board"
    )
        .then((response) => response.json())
        .then(function (data) {

            var table = document.createElement('table');
            table.style.width = '100px';
            table.style.border = "8px solid #FF9900";
            table.style.margin = "auto";
            var boardLength = data.colors.length;
            for (var i = 0; i < boardLength; i++) {
                const tr = table.insertRow();
                for (var j = 0; j < boardLength; j++) {
                    var color = colorList[data.colors[i][j]];
                    var amount = data.amounts[i][j];
                    var td = tr.insertCell(j);
                    td.appendChild(document.createTextNode(amount));
                    td.style.color = "white";
                    td.style.webkitTextStroke = "2px black"
                    td.style.fontSize = "48px";
                    td.style.fontFamily = '"Lucida Console", "Courier New", monospace';
                    td.style.backgroundColor = color;
                    td.style.padding = "30px";
                    td.style.border = '5px solid white';
                }
            }
            document.getElementById("board").innerHTML = table.outerHTML;
        });
}

function unlock(token, color) {
    get_board()
    if (document.getElementById("token").value) {
        var token_input = document.getElementById("token").value;
        document.getElementById("token").value = "";
    } else {
        token_input = "";
    }
    document.getElementById("response").innerHTML = "";
    fetch(
        "/api/land?" +
            new URLSearchParams({
                color: document.querySelector('input[name="color_choice"]:checked').value,
                token: token_input,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            get_board();
            console.log(data);
            document.getElementById("response").innerHTML = data.message;
        })
        .catch((error) => console.log(error));
    return false;
}

get_board()
get_board_interval = setInterval(function () {
    get_board();
}, 10000);