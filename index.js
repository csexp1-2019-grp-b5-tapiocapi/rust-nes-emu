import init, {start} from './pkg/nes.js';
async function run() {
    await init();
    start("sample1/sample1.nes");
}
run();

