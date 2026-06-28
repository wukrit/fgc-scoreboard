window.onload = init;

function init(){

	var urlParams = new URLSearchParams(window.location.search);
	var binId = urlParams.get('bin');
	var lanMode = !binId && window.location.protocol !== 'file:';
	var scObj; //variable to hold data extracted from parsed json
	var startup = true; //flag for if looping functions are on their first pass or not
	var animated = false; //flag for if scoreboard animation has run or not
	var game; //variable to hold game value from streamcontrol dropdown
	var currentGame = ''; //tracks current game for change detection (replaces hidden #gameHold div)

	// Cached jQuery selectors — avoid repeated DOM queries on every poll
	var $p1Name = $('#p1Name'), $p2Name = $('#p2Name');
	var $p1Team = $('#p1Team'), $p2Team = $('#p2Team');
	var $p1Score = $('#p1Score'), $p2Score = $('#p2Score');
	var $round = $('#round');
	var $p1Wrapper = $('#p1Wrapper'), $p2Wrapper = $('#p2Wrapper');

	// Game adjustment groups — data-driven lookup replaces duplicated if/else chains.
	// To add a new game, just add it to the appropriate group array.
	var GAME_GROUPS = {
		adjust1: ['BBTAG', 'SFVCE', 'TEKKEN7', 'UNICLR'],
		adjust2: ['BBCF', 'DBFZ', 'GGXRD', 'KOFXIV', 'MVCI', 'UMVC3'],
		adjust3: ['USF4'],
		logoAdjust: ['BBTAG', 'UNICLR']
	};

	// --- Helper: shrink score font until digits fit the score box ---
	function fitScoreDisplay(el, defaultSize) {
		var size = parseFloat(defaultSize);
		el.style.fontSize = size + 'px';
		var ratio = Math.min(
			el.offsetWidth / Math.max(el.scrollWidth, 1),
			el.offsetHeight / Math.max(el.scrollHeight, 1)
		);
		if (ratio < 1) {
			el.style.fontSize = Math.max(Math.floor(size * ratio), 22) + 'px';
		}
	}

	// --- Helper: shrink font until element content fits within its bounds ---
	// Uses a ratio-based approach to minimize forced reflows (2 instead of N).
	function shrinkToFit(el, defaultSize) {
		var size = parseFloat(defaultSize);
		el.style.fontSize = size + 'px';
		// Only shrink if needed, max 2 sizes down to avoid aggressive scaling
		var ratio = Math.min(
			el.offsetWidth / Math.max(el.scrollWidth, 1),
			el.offsetHeight / Math.max(el.scrollHeight, 1)
		);
		if (ratio < 1) {
			var newSize = Math.max(size - 2, Math.floor(size * ratio));
			el.style.fontSize = newSize + 'px';
		}
	}

	// --- Helper: determine which adjustment group a game belongs to ---
	function getGameGroup(gameName) {
		if (GAME_GROUPS.adjust1.indexOf(gameName) !== -1) return 'adjust1';
		if (GAME_GROUPS.adjust2.indexOf(gameName) !== -1) return 'adjust2';
		if (GAME_GROUPS.adjust3.indexOf(gameName) !== -1) return 'adjust3';
		return 'adjust2'; // default
	}

	// --- Helper: apply game-specific layout positioning ---
	function applyGameLayout(gameName, isGameChange) {
		var group = getGameGroup(gameName);

		if (group === 'adjust1') {
			var offset = document.getElementById("leftBGWrapper").offsetTop;
			TweenMax.fromTo('#leftBGWrapper', 0.5, {css:{y: offset}}, {css:{y: adjust1}});
			TweenMax.fromTo('#rightBGWrapper', 0.5, {css:{y: offset}}, {css:{y: adjust1}});
			TweenMax.set('#leftWrapper',{css:{y: '4px'}});
			TweenMax.set('#rightWrapper',{css:{y: '4px'}});
		} else if (group === 'adjust3') {
			if (isGameChange) {
				TweenMax.set('#leftWrapper',{css:{y: adjust3}});
				TweenMax.set('#rightWrapper',{css:{y: adjust3}});
			} else {
				var offset = document.getElementById("leftBGWrapper").offsetTop;
				TweenMax.fromTo('#leftBGWrapper', 0.5, {css:{y: offset}}, {css:{y: adjust3}});
				TweenMax.fromTo('#rightBGWrapper', 0.5, {css:{y: offset}}, {css:{y: adjust3}});
				TweenMax.set('#leftWrapper',{css:{y: adjust3}});
				TweenMax.set('#rightWrapper',{css:{y: adjust3}});
			}
		} else {
			// adjust2 (default)
			if (isGameChange) {
				TweenMax.set('#leftBGWrapper',{css:{y: '+0px'}});
				TweenMax.set('#rightBGWrapper',{css:{y: '+0px'}});
			}
			TweenMax.set('#leftWrapper',{css:{y: adjust2}});
			TweenMax.set('#rightWrapper',{css:{y: adjust2}});
		}

		// Logo adjustments for specific games
		if (GAME_GROUPS.logoAdjust.indexOf(gameName) !== -1) {
			var adjustLgW = parseFloat(isGameChange ? adjustLg[3] : $('.logos').css('width')) * adjustLg[2];
			var adjustLgH = parseFloat(isGameChange ? adjustLg[4] : $('.logos').css('height')) * adjustLg[2];
			TweenMax.set('.logos',{css:{x: adjustLg[0], y: adjustLg[1], width: adjustLgW, height: adjustLgH}});
		} else if (isGameChange) {
			TweenMax.set('.logos',{css:{x: '+0px', y: '+0px', width: adjustLg[3], height: adjustLg[4]}});
		}
	}

	// --- Helper: replay CSS animations by removing and re-adding classes ---
	function playCSSAnimations(){
		var animations = [
			['roundBG', 'round-animation'],
			['p1PlayerBG', 'player1-animation'],
			['p1ScoreBG', 'p1s-animation'],
			['p2PlayerBG', 'player2-animation'],
			['p2ScoreBG', 'p2s-animation']
		];
		// Batch removals
		animations.forEach(function(pair) {
			document.getElementById(pair[0]).classList.remove(pair[1]);
		});
		// Single forced reflow
		void document.getElementById('roundBG').offsetWidth;
		// Batch additions
		animations.forEach(function(pair) {
			document.getElementById(pair[0]).classList.add(pair[1]);
		});
	}

	// --- Unified polling: single factory for remote and LAN modes ---
	function createPoller(url, callback) {
		var abortCtrl = null;
		function poll() {
			if (abortCtrl) abortCtrl.abort();
			abortCtrl = new AbortController();
			fetch(url + '?v=' + Date.now(), {
				signal: abortCtrl.signal,
				// Required for loca.lt tunnels in OBS/headless browsers (no consent cookie).
				headers: { 'Bypass-Tunnel-Reminder': '1' }
			})
				.then(function(r) { return r.json(); })
				.then(function(data) { callback(data); })
				.catch(function(e) { /* ignore AbortError and network errors */ });
		}
		poll();
		setInterval(poll, 1000);
	}

	// --- Mode setup ---
	if (binId) {
		// REMOTE MODE — poll npoint.io
		console.log('REMOTE MODE');
		createPoller('https://api.npoint.io/' + binId, function(data) {
			scObj = data;
			scoreboard();
		});
		setTimeout(scoreboard, 300);

	} else if (lanMode) {
		// LAN MODE — poll local server
		console.log('LAN MODE');
		createPoller(window.location.origin + '/scoreboard.json', function(data) {
			scObj = data;
			scoreboard();
		});
		setTimeout(scoreboard, 300);

	} else {
		// LOCAL MODE — read from localStorage, sync via storage event
		console.log('LOCAL MODE');

		var stored = localStorage.getItem('fgc-scoreboard-data');
		if (stored) {
			try { scObj = JSON.parse(stored); }
			catch(e) { console.warn('Failed to parse localStorage data:', e); }
		}

		window.addEventListener('storage', function(evt) {
			if (evt.key === 'fgc-scoreboard-data' && evt.newValue) {
				try {
					scObj = JSON.parse(evt.newValue);
				} catch(e) {
					console.warn('Failed to parse localStorage data:', e);
					return;
				}
				scoreboard();
			}
		});

		if (scObj) {
			setTimeout(scoreboard, 300);
		}
	}

	function scoreboard(){
		if(!scObj) return;

		if(startup){
			game = scObj['game'];
			currentGame = game;

			applyGameLayout(game, false);
			getData();
			setTimeout(logoLoop, logoTime);
			startup = false;
			animated = true;
		}
		else{
			getData();
		}
	}

	function getData(){

		var p1Name = scObj['p1Name'];
		var p2Name = scObj['p2Name'];
		var p1Team = scObj['p1Team'];
		var p2Team = scObj['p2Team'];
		var p1Score = scObj['p1Score'];
		var p2Score = scObj['p2Score'];
		var round = scObj['round'];

		if(startup){

			TweenMax.set('#p1Wrapper',{css:{x: p1Move}});
			TweenMax.set('#p2Wrapper',{css:{x: p2Move}});
			TweenMax.set('#round',{css:{y: rdMove}});

			$p1Name.text(p1Name);
			$p2Name.text(p2Name);
			$p1Team.text(p1Team);
			$p2Team.text(p2Team);
			$p1Score.text(p1Score);
			$p2Score.text(p2Score);
			$round.text(round);

			fitScoreDisplay($p1Score[0], 34);
			fitScoreDisplay($p2Score[0], 34);
			shrinkToFit($p1Wrapper[0], nameSize);
			shrinkToFit($p2Wrapper[0], nameSize);
			shrinkToFit($round[0], rdSize);

			TweenMax.to('#p1Wrapper',nameTime,{css:{x: '+0px', opacity: 1},ease:Quad.easeOut,delay:nameDelay});
			TweenMax.to('#p2Wrapper',nameTime,{css:{x: '+0px', opacity: 1},ease:Quad.easeOut,delay:nameDelay});
			TweenMax.to('#round',rdTime,{css:{y: '+0px', opacity: 1},ease:Quad.easeOut,delay:rdDelay});
			TweenMax.to('.scores',scTime,{css:{opacity: 1},ease:Quad.easeOut,delay:scDelay});
		}
		else{
			game = scObj['game'];

			if($p1Name.text() !== p1Name || $p1Team.text() !== p1Team){
				TweenMax.killTweensOf('#p1Wrapper');
				TweenMax.to('#p1Wrapper',.3,{css:{x: p1Move, opacity: 0},ease:Quad.easeOut,delay:0,onComplete:function(){
					$p1Name.text(p1Name);
					$p1Team.text(p1Team);
					shrinkToFit($p1Wrapper[0], nameSize);
					TweenMax.to('#p1Wrapper',.3,{css:{x: '+0px', opacity: 1},ease:Quad.easeOut,delay:.2});
				}});
			}

			if($p2Name.text() !== p2Name || $p2Team.text() !== p2Team){
				TweenMax.killTweensOf('#p2Wrapper');
				TweenMax.to('#p2Wrapper',.3,{css:{x: p2Move, opacity: 0},ease:Quad.easeOut,delay:0,onComplete:function(){
					$p2Name.text(p2Name);
					$p2Team.text(p2Team);
					shrinkToFit($p2Wrapper[0], nameSize);
					TweenMax.to('#p2Wrapper',.3,{css:{x: '+0px', opacity: 1},ease:Quad.easeOut,delay:.2});
				}});
			}

			if($round.text() !== round){
				TweenMax.killTweensOf('#round');
				TweenMax.to('#round',.3,{css:{opacity: 0},ease:Quad.easeOut,delay:0,onComplete:function(){
					$round.text(round);
					shrinkToFit($round[0], rdSize);
					TweenMax.to('#round',.3,{css:{opacity: 1},ease:Quad.easeOut,delay:.2});
				}});
			}

			if($p1Score.text() !== p1Score){
				TweenMax.killTweensOf('#p1Score');
				TweenMax.to('#p1Score',.3,{css:{opacity: 0},ease:Quad.easeOut,delay:0,onComplete:function(){
					$p1Score.text(p1Score);
					fitScoreDisplay($p1Score[0], 34);
					TweenMax.to('#p1Score',.3,{css:{opacity: 1},ease:Quad.easeOut,delay:.2});
				}});
			}

			if($p2Score.text() !== p2Score){
				TweenMax.killTweensOf('#p2Score');
				TweenMax.to('#p2Score',.3,{css:{opacity: 0},ease:Quad.easeOut,delay:0,onComplete:function(){
					$p2Score.text(p2Score);
					fitScoreDisplay($p2Score[0], 34);
					TweenMax.to('#p2Score',.3,{css:{opacity: 1},ease:Quad.easeOut,delay:.2});
				}});
			}

			if(currentGame !== game){
				currentGame = game;
				TweenMax.to('#scoreboardBG',.3,{css:{opacity: 0},delay:0});
				TweenMax.to('#scoreboard',.3,{css:{opacity: 0},delay:0});
				TweenMax.to('.logos',.3,{css:{opacity: 0},delay:0,onComplete:function(){
					applyGameLayout(game, true);
					playCSSAnimations();
					TweenMax.to('#scoreboardBG',.3,{css:{opacity: 1},delay:.3});
					TweenMax.to('#scoreboard',.3,{css:{opacity: 1},delay:.3});
					TweenMax.to('.logos',.3,{css:{opacity: .7},delay:.3});
				}});
			}
		}
	}

	function logoLoop(){
		var initialTime = 700;
		var intervalTime = 15000;
		var fadeTime = 2000;
		var currentItem = 0;
		var $logos = $('#logoWrapper').find('img');
		var itemCount = $logos.length;

		if(itemCount > 1){
			$logos.eq(currentItem).fadeIn(initialTime);

			setInterval(function(){
				$logos.eq(currentItem).fadeOut(fadeTime);

				if(currentItem === itemCount - 1){
					currentItem = 0;
				}
				else{
					currentItem++;
				}

				$logos.eq(currentItem).fadeIn(fadeTime);
			},intervalTime);
		}
		else{
			$('.logos').fadeIn(initialTime);
		}
	}
}
