(()=>{var e,t,r,o,n,a,i,s={},c={};function l(e){if(c[e])return c[e].exports;var t=c[e]={id:e,loaded:!1,exports:{}};return s[e](t,t.exports,l),t.loaded=!0,t.exports}l.m=s,e="function"==typeof Symbol?Symbol("webpack then"):"__webpack_then__",t="function"==typeof Symbol?Symbol("webpack exports"):"__webpack_exports__",r=e=>{e&&(e.forEach((e=>e.r--)),e.forEach((e=>e.r--?e.r++:e())))},o=e=>!--e.r&&e(),n=(e,t)=>e?e.push(t):o(t),l.a=(a,i,s)=>{var c,l,p,u=s&&[],d=a.exports,h=!0,f=!1,b=(t,r,o)=>{f||(f=!0,r.r+=t.length,t.map(((t,n)=>{t[e](r,o)})),f=!1)},m=new Promise(((e,t)=>{p=t,l=()=>{e(d),r(u),u=0}}));m[t]=d,m[e]=(e,t)=>{if(h)return o(e);c&&b(c,e,t),n(u,e),m.catch(t)},a.exports=m,i((a=>{if(!a)return l();var i,s;c=(a=>a.map((a=>{if(null!==a&&"object"==typeof a){if(a[e])return a;if(a.then){var i=[];a.then((e=>{s[t]=e,r(i),i=0}));var s={[e]:(e,t)=>{n(i,e),a.catch(t)}};return s}}return{[e]:e=>{o(e)},[t]:a}})))(a);var p=new Promise(((e,r)=>{(i=()=>e(s=c.map((e=>e[t])))).r=0,b(c,i,r)}));return i.r?p:s})).then(l,p),h=!1},l.d=(e,t)=>{for(var r in t)l.o(t,r)&&!l.o(e,r)&&Object.defineProperty(e,r,{enumerable:!0,get:t[r]})},l.f={},l.e=e=>Promise.all(Object.keys(l.f).reduce(((t,r)=>(l.f[r](e,t),t)),[])),l.u=e=>e+".js",l.g=function(){if("object"==typeof globalThis)return globalThis;try{return this||new Function("return this")()}catch(e){if("object"==typeof window)return window}}(),l.hmd=e=>((e=Object.create(e)).children||(e.children=[]),Object.defineProperty(e,"exports",{enumerable:!0,set:()=>{throw new Error("ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: "+e.id)}}),e),l.o=(e,t)=>Object.prototype.hasOwnProperty.call(e,t),a={},i="homotopy-web:",l.l=(e,t,r,o)=>{if(a[e])a[e].push(t);else{var n,s;if(void 0!==r)for(var c=document.getElementsByTagName("script"),p=0;p<c.length;p++){var u=c[p];if(u.getAttribute("src")==e||u.getAttribute("data-webpack")==i+r){n=u;break}}n||(s=!0,(n=document.createElement("script")).charset="utf-8",n.timeout=120,l.nc&&n.setAttribute("nonce",l.nc),n.setAttribute("data-webpack",i+r),n.src=e),a[e]=[t];var d=(t,r)=>{n.onerror=n.onload=null,clearTimeout(h);var o=a[e];if(delete a[e],n.parentNode&&n.parentNode.removeChild(n),o&&o.forEach((e=>e(r))),t)return t(r)},h=setTimeout(d.bind(null,void 0,{type:"timeout",target:n}),12e4);n.onerror=d.bind(null,n.onerror),n.onload=d.bind(null,n.onload),s&&document.head.appendChild(n)}},l.r=e=>{"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},(()=>{var e;l.g.importScripts&&(e=l.g.location+"");var t=l.g.document;if(!e&&t&&(t.currentScript&&(e=t.currentScript.src),!e)){var r=t.getElementsByTagName("script");r.length&&(e=r[r.length-1].src)}if(!e)throw new Error("Automatic publicPath is not supported in this browser");e=e.replace(/#.*$/,"").replace(/\?.*$/,"").replace(/\/[^\/]+$/,"/"),l.p=e})(),(()=>{var e={826:0};l.f.j=(t,r)=>{var o=l.o(e,t)?e[t]:void 0;if(0!==o)if(o)r.push(o[2]);else{var n=new Promise(((r,n)=>{o=e[t]=[r,n]}));r.push(o[2]=n);var a=l.p+l.u(t),i=new Error;l.l(a,(r=>{if(l.o(e,t)&&(0!==(o=e[t])&&(e[t]=void 0),o)){var n=r&&("load"===r.type?"missing":r.type),a=r&&r.target&&r.target.src;i.message="Loading chunk "+t+" failed.\n("+n+": "+a+")",i.name="ChunkLoadError",i.type=n,i.request=a,o[1](i)}}),"chunk-"+t,t)}};var t=(t,r)=>{for(var o,n,[a,i,s]=r,c=0,p=[];c<a.length;c++)n=a[c],l.o(e,n)&&e[n]&&p.push(e[n][0]),e[n]=0;for(o in i)l.o(i,o)&&(l.m[o]=i[o]);for(s&&s(l),t&&t(r);p.length;)p.shift()()},r=self.webpackChunkhomotopy_web=self.webpackChunkhomotopy_web||[];r.forEach(t.bind(null,0)),r.push=t.bind(null,r.push.bind(r))})(),l.v=(e,t,r,o)=>{var n=fetch(l.p+""+r+".module.wasm");return"function"==typeof WebAssembly.instantiateStreaming?WebAssembly.instantiateStreaming(n,o).then((t=>Object.assign(e,t.instance.exports))):n.then((e=>e.arrayBuffer())).then((e=>WebAssembly.instantiate(e,o))).then((t=>Object.assign(e,t.instance.exports)))},l.e(235).then(l.bind(l,235)).catch(console.error)})();