// for-in object benchmark - tests key iteration setup and repeated key access
var obj = {
    k0: 0,  k1: 1,  k2: 2,  k3: 3,  k4: 4,
    k5: 5,  k6: 6,  k7: 7,  k8: 8,  k9: 9,
    k10: 10, k11: 11, k12: 12, k13: 13, k14: 14,
    k15: 15, k16: 16, k17: 17, k18: 18, k19: 19
};

var sum = 0;
for (var round = 0; round < 2000; round = round + 1) {
    for (var k in obj) {
        sum = sum + k.length;
    }
}

print('for_in_object sum = ' + sum);
