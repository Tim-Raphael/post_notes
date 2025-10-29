export default class Module {
    id;
    initModule;

    constructor(id, initModule) {
        this.id = id;
        this.initModule = initModule;
    }

    run() {
        this.initModule();
    }
}
