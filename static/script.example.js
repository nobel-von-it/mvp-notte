document.addEventListener("DOMContentLoaded", async () => {
    const explorer = document.getElementById("explorer");
    const editor = document.getElementById("current-file-content");

    async function lsDir(path) {
        return fetch("http://localhost:8080/api/ls", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                path: path
            })
        }).then(response => {
            response.json().then(data => {
                return data.content;
            })
        }).catch(error => {
            console.error(error)
        })
    }

    function expandDirInExplorer(dirName) {
        explorer.elements.forEach(element => {
            if (element.textContent === dirName) {
                element.classList.add("dir-entry");
                ls(dirName).forEach(file => {
                    const li = document.createElement("li");
                    li.classList.add("explorer-entry");

                    const btn = document.createElement("button");
                    btn.textContent = file;
                    btn.addEventListener("click", () => {
                        openEditor(file);
                    })
                    li.appendChild(btn);
                    element.appendChild(li);
                })
            }
        })
    }

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
                if (data.type === "file") {
                    editor.value = data.content.join("\n");
                } else if (data.type === "dir") {
                    expandDirInExplorer(data.path);
                }
            })
        }).catch(error => {
            console.error(error)
        })
    }

    function saveEditor(path) {
        fetch("http://localhost:8080/api/save", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                path: path,
                content: editor.value
            })
        }).then(response => {
            response.json().then(data => {
                console.log(data);
            })
        }).catch(error => {
            console.error(error)
        })
    }

    function dataToList(content) {
        // TODO
        return null
    }

    const startDir = ".";
    lsDir(startDir).then(files => files.forEach(file => {
        const li = document.createElement("li");
        li.classList.add("explorer-entry");

        const btn = document.createElement("button");
        btn.textContent = file;
        btn.addEventListener("click", () => {
            openEditor(file);
        })

        li.appendChild(btn);
        explorer.appendChild(li);
    }))

    openEditor("../static/index.example.html");
})


