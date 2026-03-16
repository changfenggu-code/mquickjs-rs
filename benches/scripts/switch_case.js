// switch/case benchmark - tests multi-branch strict-equality dispatch
var sum = 0;
for (var i = 0; i < 20000; i = i + 1) {
    switch (i % 10) {
        case 0: sum = sum + 10; break;
        case 1: sum = sum + 20; break;
        case 2: sum = sum + 30; break;
        case 3: sum = sum + 40; break;
        case 4: sum = sum + 50; break;
        case 5: sum = sum + 60; break;
        case 6: sum = sum + 70; break;
        case 7: sum = sum + 80; break;
        case 8: sum = sum + 90; break;
        default: sum = sum + 5;
    }
}
print("switch_case sum = " + sum);
