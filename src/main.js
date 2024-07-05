const { invoke } = window.__TAURI__.tauri;
const { open } = window.__TAURI__.dialog;

async function listZipContents(zipFilePath, password = null) {
    try {
        console.log(`Requesting contents for ZIP file at path: ${zipFilePath}`);
        const contents = await invoke('list_contents', { zipFile: zipFilePath, password: password });
        console.log('Contents received from Rust:', contents);
        return contents;
    } catch (error) {
        console.error('Error listing ZIP contents:', error);
        alert('Error listing ZIP contents: ' + error);
        throw error;
    }
}

async function handleFileSelection() {
    try {
        const selectedFile = await open({
            filters: [{ name: 'ZIP Files', extensions: ['zip'] }]
        });
        if (!selectedFile) {
            console.log('No file selected');
            return;
        }

        console.log(`Selected file path: ${selectedFile}`);
        const contents = await listZipContents(selectedFile);

        displayContents(contents);

        await updateRecentFiles(selectedFile);
    } catch (error) {
        console.error('Error handling file selection:', error);
    }
}

function buildTree(paths) {
    let root = {};

    paths.forEach(path => {
        const parts = path.split('/');
        let current = root;

        parts.forEach((part, index) => {
            if (!current[part]) {
                current[part] = (index === parts.length - 1) ? null : {};
            }
            current = current[part];
        });
    });

    return root;
}

function displayTree(element, tree) {
    const ul = document.createElement("ul");

    for (let key in tree) {
        const li = document.createElement("li");
        li.textContent = key;

        if (tree[key] !== null) {
            displayTree(li, tree[key]);
        }

        ul.appendChild(li);
    }

    element.appendChild(ul);
}

function displayContents(contents) {
    const zipContentsElement = document.getElementById("zipContents");
    zipContentsElement.innerHTML = "";

    const paths = contents.map(file => file.path);
    const tree = buildTree(paths);

    displayTree(zipContentsElement, tree);
}

async function getRecentFiles() {
    try {
        const recentFiles = await invoke('get_recent_files');
        console.log('Recent files received from Rust:', recentFiles);
        return recentFiles;
    } catch (error) {
        console.error('Error getting recent files:', error);
        alert('Error getting recent files: ' + error);
        return [];
    }
}

async function displayRecentFiles() {
    const recentFiles = await getRecentFiles();
    const recentFilesElement = document.getElementById("recentFiles");
    recentFilesElement.innerHTML = "";

    recentFiles.forEach(file => {
        const li = document.createElement("li");
        li.textContent = `${file.path} (Last accessed: ${new Date(file.timestamp).toLocaleString()})`;
        recentFilesElement.appendChild(li);
    });
}

document.addEventListener('DOMContentLoaded', function() {
    const selectFileButton = document.getElementById('selectFileButton');
    if (selectFileButton) {
        selectFileButton.addEventListener('click', handleFileSelection);
    } else {
        console.error('Element with ID "selectFileButton" not found.');
    }

    displayRecentFiles();
});

async function updateRecentFiles(selectedFile) {
    try {
        await invoke('add_recent_file', { filePath: selectedFile });
        await displayRecentFiles();
    } catch (error) {
        console.error('Error updating recent files:', error);
    }
}
