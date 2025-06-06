document.addEventListener("DOMContentLoaded", async () => {
    const explorer = document.getElementById("explorer");
    const editor = document.getElementById("current-file-content");

    async function lsDir(path) {
        try {
            const response = await fetch("http://localhost:8080/api/ls", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    path: path
                })
            });
            const data = await response.json();
            return data.content;
        } catch (error) {
            console.error(error);
            return [];
        }
    }

    async function expandDirInExplorer(dirName, dirPath) {
        for (const child of explorer.children) {
            if (child.textContent.toLowerCase() === dirName.toLowerCase()) {
                child.classList.add("dir-entry");

                const files = await lsDir(dirPath);
                for (const file of files) {
                    const li = document.createElement("li");
                    li.classList.add("explorer-entry");

                    const btn = document.createElement("button");
                    btn.textContent = file;
                    btn.addEventListener("click", () => {
                        openEditor(file);
                    })
                    li.appendChild(btn);
                    child.appendChild(li);
                }
            }
        }
    }

    async function openEditor(path) {
        try {
            console.log(path)
            const response = await fetch("http://localhost:8080/api/open", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    path: path
                })
            });

            const json_data = await response.json();

            console.log(json_data);

            const data = json_data.entry;

            if ("File" in data) {
                editor.value = data.File.content.join("\n");
            } else if ("Dir" in data) {
                expandDirInExplorer(data.Dir.name, data.Dir.path);
            } else {
                console.error("Unknown file type");
            }
        } catch (error) {
            console.error(error);
        }
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
    const startDirFiles = await lsDir(startDir)
    console.log(startDirFiles);
    for (let i = 0; i < startDirFiles.length; i++) {
        const file = startDirFiles[i];

        const li = document.createElement("li");
        li.classList.add("explorer-entry");

        const btn = document.createElement("button");
        btn.textContent = file;
        btn.addEventListener("click", () => {
            openEditor(file);
        })

        li.appendChild(btn);
        explorer.appendChild(li);
    }
    openEditor("../static/index.example.html");
})


