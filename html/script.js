function check_payment(payment_id) {
    fetch(
        "/api/check_payment?" +
            new URLSearchParams({
                payment_id: payment_id,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            console.log(data);

            if (data.payment_complete == true) {
                clearInterval(check_payment_interval);
                document.getElementById("payment_status").innerHTML = "Payment recieved!";
            }
        });
}

var check_payment_interval;
function click_pay() {
    fetch(
        "/api/create_payment?" +
            new URLSearchParams({
                amount: document.getElementById("amount").value,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            console.log(data);
            if (data.message == "payment created") {
                document.getElementById("payment_div").style.display = "inline";
                document.getElementById("address").innerHTML = data.address;
                document.getElementById("qr").style.display = "inline";
                document.getElementById("qr").src =
                    "/qr_codes/" + data.payment_id + ".png";
            } else {
                document.getElementById("payment_response").innerHTML =
                    data.message;
            }

            check_payment_interval = setInterval(function () {
                check_payment(data.payment_id);
            }, 3000);
        })
        .catch((error) => console.log(error));
    return false;
}
