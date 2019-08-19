import Rete from "rete";
import VueStringControl from "./VueStringControl.vue";

export class StringControl extends Rete.Control {
  component: any; // Fix this to the right type.
  props: any; // Fix this to the right type.
  vueContext: any;

  // @ts-ignore: Parameter 'emitter' and 'key' implicitly have an 'any' type.
  constructor(emitter, key, readonly?: any) {
    super(key);
    this.component = VueStringControl;
    this.props = { emitter, ikey: key, readonly };
  }

  // @ts-ignore: Parameter 'val' implicitly has an 'any' type.
  setValue(val) {
    this.vueContext.value = val;
  }
}
