import { readFile, writeFile } from 'node:fs/promises';

const WIDGET_URL = new URL('../public/assets/vendor/cap/widget-0.1.56/cap.min.js', import.meta.url);
const ORIGINAL_SOLVE_LIFECYCLE =
  'try{t=await v()}catch{}this.#p||(this.#p=new f(1),this.#p._spawn()),this.#p.setWasm(t);const i=e.length,s=new Array(i);let r=1,n=!1;this.#d.promoteFn=e=>{n||(n=!0,r=e,this.#p._size=e,this.#p._ensureSize(e))},null!==this.#d.pendingPromotion&&(this.#d.promoteFn(this.#d.pendingPromotion),this.#d.pendingPromotion=null);let a=0;for(;a<i;){';
const GUARDED_SOLVE_LIFECYCLE =
  'try{t=await v()}catch{}if(!this.#d)return[];this.#p||(this.#p=new f(1),this.#p._spawn()),this.#p.setWasm(t);const i=e.length,s=new Array(i);let r=1,n=!1;this.#d.promoteFn=e=>{n||(n=!0,r=e,this.#p._size=e,this.#p._ensureSize(e))},null!==this.#d.pendingPromotion&&(this.#d.promoteFn(this.#d.pendingPromotion),this.#d.pendingPromotion=null);let a=0;for(;a<i&&this.#d&&this.#p;){';

const widget = await readFile(WIDGET_URL, 'utf8');

if (!widget.includes(GUARDED_SOLVE_LIFECYCLE)) {
  const firstOccurrence = widget.indexOf(ORIGINAL_SOLVE_LIFECYCLE);
  if (firstOccurrence === -1 || firstOccurrence !== widget.lastIndexOf(ORIGINAL_SOLVE_LIFECYCLE)) {
    throw new Error('CAP widget solve lifecycle does not match the reviewed 0.1.56 asset');
  }
  await writeFile(WIDGET_URL, widget.replace(ORIGINAL_SOLVE_LIFECYCLE, GUARDED_SOLVE_LIFECYCLE));
}
