const { invoke } = window.__TAURI__.tauri;
const { open } = window.__TAURI__.dialog;

// Function to list contents of a ZIP file
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

// Function to handle file selection and interaction with backend
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

        const encryptedFiles = contents.filter(file => file.encrypted);
        if (encryptedFiles.length > 0) {
            const password = prompt('This ZIP file is encrypted. Please enter the password:');
            if (password === null) {
                console.log('Password prompt was canceled.');
                return;
            }
            const decryptedContents = await listZipContents(selectedFile, password);
            displayContents(decryptedContents);
        } else {
            displayContents(contents);
        }

        updateRecentFiles();
    } catch (error) {
        console.error('Error handling file selection:', error);
    }
}

// Function to display ZIP contents in the UI
function displayContents(contents) {
    const zipContentsElement = document.getElementById("zipContents");
    zipContentsElement.innerHTML = "";

    contents.forEach(file => {
        const li = document.createElement("li");
        li.textContent = file.name;
        zipContentsElement.appendChild(li);
    });
}

// Function to get recent files from the backend
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

// Function to display recent files in the UI
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

// Ensure DOM is fully loaded before attaching event listeners
document.addEventListener('DOMContentLoaded', function() {
    const selectFileButton = document.getElementById('selectFileButton');
    if (selectFileButton) {
        selectFileButton.addEventListener('click', handleFileSelection);
    } else {
        console.error('Element with ID "selectFileButton" not found.');
    }

    displayRecentFiles();
});

async function updateRecentFiles() {
    await displayRecentFiles();
}
