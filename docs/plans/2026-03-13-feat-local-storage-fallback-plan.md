---
title: "feat: Add localStorage fallback when no npoint.io bin provided"
type: feat
date: 2026-03-13
---

# feat: Add localStorage fallback when no npoint.io bin provided

## Overview

When the `?bin=` URL parameter is absent, both the overlay and controller should fall back to using `localStorage` instead of npoint.io. This enables offline development/testing and LAN tournament use (both pages in regular browser tabs on the same machine). Mode is auto-detected — no configuration needed.

**Not for OBS production use.** OBS browser sources run in an isolated CEF origin — localStorage won't be shared between a regular browser (controller) and OBS (overlay). Local mode is for dev/preview and LAN setups where both pages are open in regular browser tabs.

See: `docs/brainstorms/2026-03-13-local-fallback-brainstorm.md`

## Proposed Solution

Dual-mode data layer: if `?bin=` is present, use npoint.io (current behavior, unchanged). If absent, use `localStorage` with key `fgc-scoreboard-data`. The overlay listens for the `storage` event for real-time cross-tab sync instead of polling.

## Acceptance Criteria

- [x] Overlay works without `?bin=` param — reads scoreboard data from `localStorage`
- [x] Controller works without `?bin=` param — saves form data to `localStorage`
- [x] Controller pre-populates form from `localStorage` on page load in local mode
- [x] Overlay receives real-time updates via `storage` event (no polling in local mode)
- [x] Both files log `LOCAL MODE` to console when in local mode
- [x] Remote mode (`?bin=` present) is completely unchanged
- [x] Scores remain as strings in localStorage JSON
- [x] No new files, no new dependencies
- [x] `#no-bin-warning` shows "Local mode" message instead of error
- [x] localStorage errors are caught and surfaced via `showStatus()`

## MVP

### `_overlays/js/scoreboard.js`

Current init flow (lines 6-11):
```javascript
var binId = urlParams.get('bin');
if (!binId) {
    console.error('Missing required ?bin= URL parameter');
    return;
}
var streamJSON = 'https://api.npoint.io/' + binId;
```

New init flow — replace the early return with a localStorage branch.

**Critical:** `scObj` must be declared before the branch (move up from line 13). The existing `setTimeout(scoreboard, 300)` on line 94 must move inside the `else` (remote) branch — it currently runs unconditionally and would double-fire or break if left outside.

```javascript
var binId = urlParams.get('bin');
var scObj; // moved up from line 13

if (!binId) {
    // LOCAL MODE
    console.log('LOCAL MODE');

    // Read initial data from localStorage
    var stored = localStorage.getItem('fgc-scoreboard-data');
    if (stored) {
        try { scObj = JSON.parse(stored); }
        catch(e) { console.warn('Failed to parse localStorage data:', e); }
    }

    // Listen for cross-tab updates via storage event
    window.addEventListener('storage', function(evt) {
        if (evt.key === 'fgc-scoreboard-data' && evt.newValue) {
            try {
                scObj = JSON.parse(evt.newValue);
            } catch(e) {
                console.warn('Failed to parse localStorage data:', e);
                return;
            }
            // If startup already ran, update the scoreboard.
            // If startup hasn't run yet, call scoreboard() to trigger it.
            scoreboard();
        }
    });

    // Kick off first render if we have data
    if (scObj) {
        setTimeout(scoreboard, 300);
    }
    // If no data, overlay stays blank until first storage event arrives.

} else {
    // REMOTE MODE — existing behavior, unchanged
    var streamJSON = 'https://api.npoint.io/' + binId;

    // ... existing xhr, pollJSON, setInterval, parseJSON code ...

    setTimeout(scoreboard, 300); // moved from line 94 into remote branch
}
```

Key details:
- The `storage` event only fires in *other* tabs on the same origin — exactly what we want (controller writes, overlay receives)
- The storage handler calls `scoreboard()` unconditionally (not gated on `animated` flag) so it works both for first-render and subsequent updates. The `startup` flag inside `scoreboard()` already handles the first-vs-subsequent logic.
- All existing animation/getData logic (`scoreboard()`, `getData()`, `playCSSAnimations()`, `logoLoop()`) is reused as-is since it just reads from `scObj`
- Use `evt` for the StorageEvent parameter to avoid shadowing `e` in the catch block

### `controller.html`

**Extract `populateForm(data)` helper** to avoid duplicating 8 lines between remote and local load paths:

```javascript
// controller.html — new helper (called from both remote and local load)
function populateForm(data) {
    document.getElementById('p1Name').value = data.p1Name || '';
    document.getElementById('p1Team').value = data.p1Team || '';
    document.getElementById('p1Score').value = data.p1Score || '0';
    document.getElementById('p2Name').value = data.p2Name || '';
    document.getElementById('p2Team').value = data.p2Team || '';
    document.getElementById('p2Score').value = data.p2Score || '0';
    document.getElementById('round').value = data.round || '';
    document.getElementById('game').value = data.game || '';
}
```

Mode detection — replace warning with local mode info:

```javascript
var binId = urlParams.get('bin');
var apiUrl = binId ? 'https://api.npoint.io/' + binId : null;
var localMode = !apiUrl;

if (localMode) {
    console.log('LOCAL MODE');
    // Repurpose warning banner as local mode indicator
    var warning = document.getElementById('no-bin-warning');
    warning.textContent = 'Local mode \u2014 data saved to this browser only.';
    warning.style.display = 'block';
}
```

Load block — add localStorage branch using `populateForm`:

```javascript
if (apiUrl) {
    showStatus('Loading...', 'loading');
    fetch(apiUrl)
        .then(function(r) { return r.json(); })
        .then(function(data) {
            if (data.p1Name !== undefined) populateForm(data);
            showStatus('Loaded', 'ok');
        })
        .catch(function() {
            showStatus('Could not load data from bin', 'err');
        });
} else {
    // Local mode: load from localStorage
    var stored = localStorage.getItem('fgc-scoreboard-data');
    if (stored) {
        try {
            populateForm(JSON.parse(stored));
        } catch(e) {
            console.warn('Failed to parse localStorage data:', e);
        }
    }
}
```

Save function — add localStorage branch with error handling:

```javascript
function save() {
    var formData = getFormData();
    if (localMode) {
        try {
            localStorage.setItem('fgc-scoreboard-data', JSON.stringify(formData));
            showStatus('Saved (local)', 'ok');
        } catch(e) {
            showStatus('Save failed: ' + e.message, 'err');
        }
        return;
    }
    // ... existing npoint.io POST unchanged ...
}
```

### Changes to `#no-bin-warning`

Repurpose (not remove) the existing warning div. In local mode, change its text to "Local mode — data saved to this browser only." and display it. This prevents user confusion about where data is going without adding new UI elements. Keep the existing CSS styling but could optionally change background color from yellow to a neutral blue/gray.

## Edge Cases

- **First use (empty localStorage):** Controller starts with empty form, overlay stays blank until first save. Acceptable — same as a fresh npoint.io bin.
- **`storage` event only fires cross-tab:** If controller and overlay are in the same tab (unlikely but possible via iframes), the event won't fire. Fine for the intended two-tab workflow.
- **Chrome `file://` origin isolation:** Chrome treats each `file://` page as a separate origin — localStorage won't be shared. Users must serve files via `python3 -m http.server` or use Firefox. Document this.
- **OBS browser source:** Runs in an isolated CEF origin. Local mode won't work with OBS. This is by design — local mode is for dev/preview, not production streaming.
- **Data shape:** Uses the same `getFormData()` output including `cTitle1`, `cTitle2`, `mText1-4`, and `timestamp` fields. These are harmless extras that maintain compatibility.
- **Swap/Reset/Clear:** Still require explicit Save to update localStorage — consistent with remote mode behavior.
- **localStorage blocked (incognito/privacy mode):** The `try/catch` around `setItem` in the save function surfaces the error via `showStatus()`.

## References

- Brainstorm: `docs/brainstorms/2026-03-13-local-fallback-brainstorm.md`
- Current overlay logic: `_overlays/js/scoreboard.js:1-46`
- Current controller logic: `controller.html:234-304`
- MDN Storage event: https://developer.mozilla.org/en-US/docs/Web/API/Window/storage_event
