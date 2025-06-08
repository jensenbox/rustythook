// RustyHook Documentation Custom JavaScript

// Wait for the DOM to be fully loaded
document.addEventListener('DOMContentLoaded', function() {
    // Add version selector if available
    addVersionSelector();
    
    // Add copy buttons to code blocks
    addCopyButtons();
    
    // Add anchor links to headings
    addAnchorLinks();
    
    // Add note/warning/tip classes to blockquotes with specific prefixes
    processBlockquotes();
});

// Add a version selector dropdown if multiple versions are available
function addVersionSelector() {
    // This would typically be populated from a versions.json file
    const versions = [
        { name: 'latest', url: '/rustyhook/' },
        // Add more versions as they become available
    ];
    
    if (versions.length <= 1) return;
    
    const nav = document.querySelector('.nav-chapters');
    if (!nav) return;
    
    const selector = document.createElement('div');
    selector.className = 'version-selector';
    selector.innerHTML = `
        <label for="version-select">Version:</label>
        <select id="version-select">
            ${versions.map(v => `<option value="${v.url}">${v.name}</option>`).join('')}
        </select>
    `;
    
    nav.parentNode.insertBefore(selector, nav);
    
    document.getElementById('version-select').addEventListener('change', function() {
        window.location.href = this.value;
    });
}

// Add copy buttons to code blocks
function addCopyButtons() {
    const codeBlocks = document.querySelectorAll('pre > code');
    
    codeBlocks.forEach(function(codeBlock) {
        const container = codeBlock.parentNode;
        
        // Create copy button
        const copyButton = document.createElement('button');
        copyButton.className = 'copy-button';
        copyButton.textContent = 'Copy';
        
        // Add copy functionality
        copyButton.addEventListener('click', function() {
            const code = codeBlock.textContent;
            navigator.clipboard.writeText(code).then(function() {
                copyButton.textContent = 'Copied!';
                setTimeout(function() {
                    copyButton.textContent = 'Copy';
                }, 2000);
            }).catch(function(error) {
                console.error('Failed to copy code: ', error);
                copyButton.textContent = 'Error!';
                setTimeout(function() {
                    copyButton.textContent = 'Copy';
                }, 2000);
            });
        });
        
        // Add button to container
        container.appendChild(copyButton);
    });
}

// Add anchor links to headings
function addAnchorLinks() {
    const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
    
    headings.forEach(function(heading) {
        if (heading.id) {
            const anchor = document.createElement('a');
            anchor.className = 'anchor-link';
            anchor.href = `#${heading.id}`;
            anchor.innerHTML = '#';
            anchor.title = 'Permalink to this section';
            
            heading.appendChild(anchor);
        }
    });
}

// Process blockquotes to add note/warning/tip classes
function processBlockquotes() {
    const blockquotes = document.querySelectorAll('blockquote');
    
    blockquotes.forEach(function(blockquote) {
        const firstParagraph = blockquote.querySelector('p:first-child');
        if (!firstParagraph) return;
        
        const text = firstParagraph.textContent;
        
        if (text.startsWith('Note:')) {
            blockquote.className = 'note';
            firstParagraph.innerHTML = firstParagraph.innerHTML.replace('Note:', '<strong>Note:</strong>');
        } else if (text.startsWith('Warning:')) {
            blockquote.className = 'warning';
            firstParagraph.innerHTML = firstParagraph.innerHTML.replace('Warning:', '<strong>Warning:</strong>');
        } else if (text.startsWith('Tip:')) {
            blockquote.className = 'tip';
            firstParagraph.innerHTML = firstParagraph.innerHTML.replace('Tip:', '<strong>Tip:</strong>');
        }
    });
}