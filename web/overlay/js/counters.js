window.onload = init;

function init() {
	var urlParams = new URLSearchParams(window.location.search);
	var binId = urlParams.get('bin');
	var lanMode = !binId && window.location.protocol !== 'file:';
	var scObj;
	var startup = true;
	var renderedIds = [];

	function createPoller(url, callback) {
		var abortCtrl = null;
		function poll() {
			if (abortCtrl) abortCtrl.abort();
			abortCtrl = new AbortController();
			fetch(url + '?v=' + Date.now(), {
				signal: abortCtrl.signal,
				headers: { 'Bypass-Tunnel-Reminder': '1' }
			})
				.then(function(r) { return r.json(); })
				.then(function(data) { callback(data); })
				.catch(function() { /* silent retry */ });
		}
		poll();
		setInterval(poll, 1000);
	}

	function getCounters(scObj) {
		if (!scObj || !scObj.counters || typeof scObj.counters !== 'object') {
			return {};
		}
		return scObj.counters;
	}

	var LABEL_MAX_SIZE = 20;
	var LABEL_MIN_SIZE = 10;
	var LABEL_LINE_HEIGHT = 1.15;
	var LABEL_MAX_LINES = 2;

	function shrinkLabelToFit(el, wrapEl) {
		var maxHeight = wrapEl.clientHeight;
		var maxWidth = wrapEl.clientWidth;
		var size = LABEL_MAX_SIZE;

		el.style.fontSize = size + 'px';
		el.style.maxHeight = (size * LABEL_LINE_HEIGHT * LABEL_MAX_LINES) + 'px';

		while (size > LABEL_MIN_SIZE && (el.scrollWidth > maxWidth || el.scrollHeight > maxHeight)) {
			size -= 1;
			el.style.fontSize = size + 'px';
			el.style.maxHeight = (size * LABEL_LINE_HEIGHT * LABEL_MAX_LINES) + 'px';
		}
	}

	function fitCounterLabel($card) {
		var el = $card.find('.counter-label')[0];
		var wrap = $card.find('.counter-label-wrap')[0];
		if (!el || !wrap) return;
		el.style.fontSize = '';
		el.style.maxHeight = '';
		shrinkLabelToFit(el, wrap);
	}

	function fitAllCounterLabels() {
		requestAnimationFrame(function() {
			$('#counters-bar .counter-card').each(function() {
				fitCounterLabel($(this));
			});
		});
	}

	function buildCard(id, label, value) {
		var card = document.createElement('div');
		card.className = 'counter-card counter-intro';
		card.setAttribute('role', 'listitem');
		card.setAttribute('data-counter-id', id);
		card.innerHTML =
			'<div class="counter-label-wrap">' +
				'<span class="counter-label">' + escapeText(label || 'Counter') + '</span>' +
			'</div>' +
			'<div class="counter-value-wrap">' +
				'<span class="counter-value" id="counter-val-' + id + '">' + escapeText(value || '0') + '</span>' +
			'</div>';
		return card;
	}

	function escapeText(str) {
		return String(str)
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;');
	}

	function animateValueChange($el, newVal) {
		TweenMax.killTweensOf($el);
		TweenMax.to($el, 0.3, {
			css: { opacity: 0 },
			ease: Quad.easeOut,
			delay: 0,
			onComplete: function() {
				$el.text(newVal);
				TweenMax.to($el, 0.3, { css: { opacity: 1 }, ease: Quad.easeOut, delay: 0.2 });
			}
		});
	}

	function renderCounters() {
		var counters = getCounters(scObj);
		var ids = Object.keys(counters);
		var $bar = $('#counters-bar');

		if (ids.length === 0) {
			$bar.empty();
			renderedIds = [];
			return;
		}

		var idsKey = ids.slice().sort().join(',');
		var prevKey = renderedIds.slice().sort().join(',');

		if (idsKey !== prevKey) {
			$bar.empty();
			ids.forEach(function(id) {
				var c = counters[id] || {};
				$bar.append(buildCard(id, c.label, c.value || '0'));
			});
			fitAllCounterLabels();
			if (startup) {
				TweenMax.fromTo('.counter-card', 0.4, { css: { opacity: 0, y: 24 } }, { css: { opacity: 1, y: 0 }, ease: Quad.easeOut, delay: 0.15, stagger: 0.08 });
			}
			renderedIds = ids.slice();
			return;
		}

		ids.forEach(function(id) {
			var c = counters[id] || {};
			var val = c.value !== undefined && c.value !== '' ? String(c.value) : '0';
			var label = c.label || 'Counter';
			var $card = $('[data-counter-id="' + id + '"]');
			if (!$card.length) return;
			$card.find('.counter-label').text(label);
			fitCounterLabel($card);
			var $val = $('#counter-val-' + id);
			if ($val.text() !== val) {
				animateValueChange($val, val);
			}
		});
	}

	function countersUpdate() {
		if (!scObj) return;
		renderCounters();
		startup = false;
	}

	if (binId) {
		console.log('REMOTE MODE (counters)');
		createPoller('https://api.npoint.io/' + binId, function(data) {
			scObj = data;
			countersUpdate();
		});
		setTimeout(countersUpdate, 300);
	} else if (lanMode) {
		console.log('LAN MODE (counters)');
		createPoller(window.location.origin + '/scoreboard.json', function(data) {
			scObj = data;
			countersUpdate();
		});
		setTimeout(countersUpdate, 300);
	} else {
		console.log('LOCAL MODE (counters)');
		var stored = localStorage.getItem('fgc-scoreboard-data');
		if (stored) {
			try { scObj = JSON.parse(stored); }
			catch (e) { console.warn('Failed to parse localStorage data:', e); }
		}
		window.addEventListener('storage', function(evt) {
			if (evt.key === 'fgc-scoreboard-data' && evt.newValue) {
				try {
					scObj = JSON.parse(evt.newValue);
					countersUpdate();
				} catch (e) {
					console.warn('Failed to parse localStorage data:', e);
				}
			}
		});
		if (scObj) {
			setTimeout(countersUpdate, 300);
		}
	}
}
