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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction to Xmlity</a></li><li class="chapter-item expanded affix "><a href="help.html">Help</a></li><li class="chapter-item expanded affix "><li class="part-title">User Guide</li><li class="chapter-item expanded "><a href="1_getting_started/index.html"><strong aria-hidden="true">1.</strong> Getting started</a></li><li class="chapter-item expanded "><a href="2_the_data_model/index.html"><strong aria-hidden="true">2.</strong> The data model</a></li><li class="chapter-item expanded "><a href="3_using_derive/index.html"><strong aria-hidden="true">3.</strong> Using derive</a></li><li class="chapter-item expanded "><a href="4_custom_serialization/index.html"><strong aria-hidden="true">4.</strong> Custom serialization</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="4_custom_serialization/serialize.html"><strong aria-hidden="true">4.1.</strong> Implementing Serialize</a></li><li class="chapter-item expanded "><a href="4_custom_serialization/serialize_attribute.html"><strong aria-hidden="true">4.2.</strong> Implementing SerializeAttribute</a></li><li class="chapter-item expanded "><a href="4_custom_serialization/deserialize.html"><strong aria-hidden="true">4.3.</strong> Implementing Deserialize</a></li></ol></li><li class="chapter-item expanded "><a href="5_implementing_an_xml_library/index.html"><strong aria-hidden="true">5.</strong> Implementing a XML library</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="5_implementing_an_xml_library/serializer.html"><strong aria-hidden="true">5.1.</strong> Implementing a Serializer</a></li><li class="chapter-item expanded "><a href="5_implementing_an_xml_library/deserializer.html"><strong aria-hidden="true">5.2.</strong> Implementing a Deserializer</a></li></ol></li><li class="chapter-item expanded "><a href="6_examples/index.html"><strong aria-hidden="true">6.</strong> Examples</a></li><li class="chapter-item expanded "><a href="7_feature_flags/index.html"><strong aria-hidden="true">7.</strong> Feature flags</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0];
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
