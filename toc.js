// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">User Guide</li><li class="chapter-item expanded "><a href="user-guide/getting-started.html"><strong aria-hidden="true">1.</strong> Getting Started</a></li><li class="chapter-item expanded "><a href="user-guide/installation.html"><strong aria-hidden="true">2.</strong> Installation</a></li><li class="chapter-item expanded "><a href="user-guide/cli-usage.html"><strong aria-hidden="true">3.</strong> CLI Usage</a></li><li class="chapter-item expanded "><a href="user-guide/configuration.html"><strong aria-hidden="true">4.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="user-guide/migration.html"><strong aria-hidden="true">5.</strong> Migration from pre-commit</a></li><li class="chapter-item expanded "><a href="user-guide/shell-completions.html"><strong aria-hidden="true">6.</strong> Shell Completions</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference</li><li class="chapter-item expanded "><a href="reference/supported-languages.html"><strong aria-hidden="true">7.</strong> Supported Languages</a></li><li class="chapter-item expanded "><a href="reference/hook-types.html"><strong aria-hidden="true">8.</strong> Hook Types</a></li><li class="chapter-item expanded "><a href="reference/configuration-reference.html"><strong aria-hidden="true">9.</strong> Configuration Reference</a></li><li class="chapter-item expanded affix "><li class="part-title">Advanced</li><li class="chapter-item expanded "><a href="advanced/monorepo.html"><strong aria-hidden="true">10.</strong> Monorepo Support</a></li><li class="chapter-item expanded "><a href="advanced/custom-hooks.html"><strong aria-hidden="true">11.</strong> Custom Hooks</a></li><li class="chapter-item expanded "><a href="advanced/performance.html"><strong aria-hidden="true">12.</strong> Performance Tuning</a></li><li class="chapter-item expanded affix "><li class="part-title">Contributing</li><li class="chapter-item expanded "><a href="contributing/development-setup.html"><strong aria-hidden="true">13.</strong> Development Setup</a></li><li class="chapter-item expanded "><a href="contributing/architecture.html"><strong aria-hidden="true">14.</strong> Architecture</a></li><li class="chapter-item expanded "><a href="contributing/testing.html"><strong aria-hidden="true">15.</strong> Testing</a></li><li class="chapter-item expanded affix "><a href="faq.html">FAQ</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
