import Rete from "rete";
import VueNetworkingCidrBlockControl from "./VueNetworkingCidrBlockControl.vue";

export class NetworkingCidrBlockControl extends Rete.Control {
  component: any; // Fix this
  props: any; // Fix this
  vueContext: any; // Fix this

  constructor(emitter: unknown, key: string, readonly?: unknown) {
    super(key);
    this.component = VueNetworkingCidrBlockControl;
    this.props = { emitter, ikey: key, readonly };
  }

  // @ts-ignore: Parameter 'val' implicitly has an 'any' type.
  setValue(val) {
    this.vueContext.value = val;
  }
}
