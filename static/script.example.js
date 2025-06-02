document.addEventListener("DOMContentLoaded", () => {
    const explorer = document.getElementById("explorer");
    const editor = document.getElementById("current-file-content");

    function openEditor(path) {
        fetch("http://localhost:8080/api/open", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                path: path
            })
        }).then(response => {
            response.json().then(data => {
                editor.value = data.content.join("\n");
            })
        }).catch(error => {
            console.error(error)
        })
    }

    fetch("http://localhost:8080/api/ls", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            path: "."
        })
    }).then(response => {
        response.json().then(data => {
            data.content.forEach(file => {
                const li = document.createElement("li");
                li.classList.add("explorer-entry");

                const btn = document.createElement("button");
                btn.textContent = file;
                btn.addEventListener("click", () => {
                    openEditor(file);
                })

                li.appendChild(btn);
                explorer.appendChild(li);
            })
        })
    }).catch(error => {
        console.error(error)
    })

    openEditor("../static/index.example.html");
})


