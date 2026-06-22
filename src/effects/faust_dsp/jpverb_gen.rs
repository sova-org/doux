/* ------------------------------------------------------------
name: "jpverb"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn JpverbDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct JpverbDsp {
	fHslider0: F32,
	fHslider1: F32,
	fHslider2: F32,
	fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fHslider3: F32,
	iVec0: [i32;2],
	fRec15: [F32;2],
	fRec16: [F32;2],
	fHslider4: F32,
	fHslider5: F32,
	fHslider6: F32,
	fConst2: F32,
	fHslider7: F32,
	IOTA0: i32,
	fVec1: [F32;16384],
	fRec53: [F32;2],
	fVec2: [F32;2],
	fRec52: [F32;2],
	fRec50: [F32;2],
	fVec3: [F32;16384],
	fRec55: [F32;2],
	fVec4: [F32;2],
	fRec54: [F32;2],
	fRec51: [F32;2],
	fVec5: [F32;16384],
	fRec56: [F32;2],
	fVec6: [F32;2],
	fRec49: [F32;2],
	fRec47: [F32;2],
	fVec7: [F32;16384],
	fRec58: [F32;2],
	fVec8: [F32;2],
	fRec57: [F32;2],
	fRec48: [F32;2],
	fVec9: [F32;16384],
	fRec59: [F32;2],
	fVec10: [F32;2],
	fRec46: [F32;2],
	fRec44: [F32;2],
	fVec11: [F32;16384],
	fRec61: [F32;2],
	fVec12: [F32;2],
	fRec60: [F32;2],
	fRec45: [F32;2],
	fVec13: [F32;16384],
	fRec62: [F32;2],
	fVec14: [F32;2],
	fRec43: [F32;2],
	fRec41: [F32;2],
	fVec15: [F32;16384],
	fRec64: [F32;2],
	fVec16: [F32;2],
	fRec63: [F32;2],
	fRec42: [F32;2],
	fVec17: [F32;16384],
	fRec65: [F32;2],
	fVec18: [F32;2],
	fRec40: [F32;2],
	fRec38: [F32;2],
	fVec19: [F32;16384],
	fRec67: [F32;2],
	fVec20: [F32;2],
	fRec66: [F32;2],
	fRec39: [F32;2],
	fVec21: [F32;1024],
	fVec22: [F32;16384],
	fRec68: [F32;2],
	fVec23: [F32;2],
	fRec37: [F32;2],
	fVec24: [F32;1024],
	fVec25: [F32;16384],
	fRec70: [F32;2],
	fVec26: [F32;2],
	fRec69: [F32;2],
	fVec27: [F32;16384],
	fRec71: [F32;2],
	fVec28: [F32;2],
	fRec36: [F32;2],
	fRec34: [F32;2],
	fVec29: [F32;16384],
	fRec73: [F32;2],
	fVec30: [F32;2],
	fRec72: [F32;2],
	fRec35: [F32;2],
	fVec31: [F32;16384],
	fRec74: [F32;2],
	fVec32: [F32;2],
	fRec33: [F32;2],
	fRec31: [F32;2],
	fVec33: [F32;16384],
	fRec76: [F32;2],
	fVec34: [F32;2],
	fRec75: [F32;2],
	fRec32: [F32;2],
	fVec35: [F32;16384],
	fRec77: [F32;2],
	fVec36: [F32;2],
	fRec30: [F32;2],
	fRec28: [F32;2],
	fVec37: [F32;16384],
	fRec79: [F32;2],
	fVec38: [F32;2],
	fRec78: [F32;2],
	fRec29: [F32;2],
	fVec39: [F32;16384],
	fRec80: [F32;2],
	fVec40: [F32;2],
	fRec27: [F32;2],
	fRec25: [F32;2],
	fVec41: [F32;16384],
	fRec82: [F32;2],
	fVec42: [F32;2],
	fRec81: [F32;2],
	fRec26: [F32;2],
	fVec43: [F32;16384],
	fRec83: [F32;2],
	fVec44: [F32;2],
	fRec24: [F32;2],
	fRec22: [F32;2],
	fVec45: [F32;16384],
	fRec85: [F32;2],
	fVec46: [F32;2],
	fRec84: [F32;2],
	fRec23: [F32;2],
	fVec47: [F32;16384],
	fVec48: [F32;16384],
	fRec86: [F32;2],
	fVec49: [F32;2],
	fRec21: [F32;2],
	fRec20: [F32;2],
	fRec19: [F32;3],
	fRec18: [F32;3],
	fHslider8: F32,
	fRec17: [F32;3],
	fRec92: [F32;2],
	fRec91: [F32;3],
	fRec90: [F32;3],
	fVec50: [F32;2],
	fRec89: [F32;2],
	fRec88: [F32;3],
	fRec87: [F32;3],
	fHslider9: F32,
	fRec95: [F32;2],
	fRec94: [F32;3],
	fRec93: [F32;3],
	fVec51: [F32;1024],
	fRec14: [F32;2],
	fVec52: [F32;16384],
	fVec53: [F32;16384],
	fRec102: [F32;2],
	fVec54: [F32;2],
	fRec101: [F32;2],
	fRec100: [F32;2],
	fRec99: [F32;3],
	fRec98: [F32;3],
	fRec97: [F32;3],
	fRec108: [F32;2],
	fRec107: [F32;3],
	fRec106: [F32;3],
	fVec55: [F32;2],
	fRec105: [F32;2],
	fRec104: [F32;3],
	fRec103: [F32;3],
	fRec111: [F32;2],
	fRec110: [F32;3],
	fRec109: [F32;3],
	fVec56: [F32;1024],
	fRec96: [F32;2],
	fVec57: [F32;16384],
	fVec58: [F32;2],
	fRec13: [F32;2],
	fRec11: [F32;2],
	fVec59: [F32;16384],
	fRec113: [F32;2],
	fVec60: [F32;2],
	fRec112: [F32;2],
	fRec12: [F32;2],
	fVec61: [F32;16384],
	fVec62: [F32;2],
	fRec10: [F32;2],
	fRec8: [F32;2],
	fVec63: [F32;16384],
	fVec64: [F32;2],
	fRec114: [F32;2],
	fRec9: [F32;2],
	fVec65: [F32;16384],
	fVec66: [F32;2],
	fRec7: [F32;2],
	fRec5: [F32;2],
	fVec67: [F32;16384],
	fRec116: [F32;2],
	fVec68: [F32;2],
	fRec115: [F32;2],
	fRec6: [F32;2],
	fVec69: [F32;16384],
	fRec117: [F32;2],
	fVec70: [F32;2],
	fRec4: [F32;2],
	fRec2: [F32;2],
	fVec71: [F32;16384],
	fVec72: [F32;2],
	fRec118: [F32;2],
	fRec3: [F32;2],
	fRec0: [F32;2],
	fRec1: [F32;2],
}

pub type FaustFloat = F32;
static iJpverbDspSIG0Wave0: [i32;2048] = [2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,101,103,107,109,113,127,131,137,139,149,151,157,163,167,173,179,181,191,193,197,199,211,223,227,229,233,239,241,251,257,263,269,271,277,281,283,293,307,311,313,317,331,337,347,349,353,359,367,373,379,383,389,397,401,409,419,421,431,433,439,443,449,457,461,463,467,479,487,491,499,503,509,521,523,541,547,557,563,569,571,577,587,593,599,601,607,613,617,619,631,641,643,647,653,659,661,673,677,683,691,701,709,719,727,733,739,743,751,757,761,769,773,787,797,809,811,821,823,827,829,839,853,857,859,863,877,881,883,887,907,911,919,929,937,941,947,953,967,971,977,983,991,997,1009,1013,1019,1021,1031,1033,1039,1049,1051,1061,1063,1069,1087,1091,1093,1097,1103,1109,1117,1123,1129,1151,1153,1163,1171,1181,1187,1193,1201,1213,1217,1223,1229,1231,1237,1249,1259,1277,1279,1283,1289,1291,1297,1301,1303,1307,1319,1321,1327,1361,1367,1373,1381,1399,1409,1423,1427,1429,1433,1439,1447,1451,1453,1459,1471,1481,1483,1487,1489,1493,1499,1511,1523,1531,1543,1549,1553,1559,1567,1571,1579,1583,1597,1601,1607,1609,1613,1619,1621,1627,1637,1657,1663,1667,1669,1693,1697,1699,1709,1721,1723,1733,1741,1747,1753,1759,1777,1783,1787,1789,1801,1811,1823,1831,1847,1861,1867,1871,1873,1877,1879,1889,1901,1907,1913,1931,1933,1949,1951,1973,1979,1987,1993,1997,1999,2003,2011,2017,2027,2029,2039,2053,2063,2069,2081,2083,2087,2089,2099,2111,2113,2129,2131,2137,2141,2143,2153,2161,2179,2203,2207,2213,2221,2237,2239,2243,2251,2267,2269,2273,2281,2287,2293,2297,2309,2311,2333,2339,2341,2347,2351,2357,2371,2377,2381,2383,2389,2393,2399,2411,2417,2423,2437,2441,2447,2459,2467,2473,2477,2503,2521,2531,2539,2543,2549,2551,2557,2579,2591,2593,2609,2617,2621,2633,2647,2657,2659,2663,2671,2677,2683,2687,2689,2693,2699,2707,2711,2713,2719,2729,2731,2741,2749,2753,2767,2777,2789,2791,2797,2801,2803,2819,2833,2837,2843,2851,2857,2861,2879,2887,2897,2903,2909,2917,2927,2939,2953,2957,2963,2969,2971,2999,3001,3011,3019,3023,3037,3041,3049,3061,3067,3079,3083,3089,3109,3119,3121,3137,3163,3167,3169,3181,3187,3191,3203,3209,3217,3221,3229,3251,3253,3257,3259,3271,3299,3301,3307,3313,3319,3323,3329,3331,3343,3347,3359,3361,3371,3373,3389,3391,3407,3413,3433,3449,3457,3461,3463,3467,3469,3491,3499,3511,3517,3527,3529,3533,3539,3541,3547,3557,3559,3571,3581,3583,3593,3607,3613,3617,3623,3631,3637,3643,3659,3671,3673,3677,3691,3697,3701,3709,3719,3727,3733,3739,3761,3767,3769,3779,3793,3797,3803,3821,3823,3833,3847,3851,3853,3863,3877,3881,3889,3907,3911,3917,3919,3923,3929,3931,3943,3947,3967,3989,4001,4003,4007,4013,4019,4021,4027,4049,4051,4057,4073,4079,4091,4093,4099,4111,4127,4129,4133,4139,4153,4157,4159,4177,4201,4211,4217,4219,4229,4231,4241,4243,4253,4259,4261,4271,4273,4283,4289,4297,4327,4337,4339,4349,4357,4363,4373,4391,4397,4409,4421,4423,4441,4447,4451,4457,4463,4481,4483,4493,4507,4513,4517,4519,4523,4547,4549,4561,4567,4583,4591,4597,4603,4621,4637,4639,4643,4649,4651,4657,4663,4673,4679,4691,4703,4721,4723,4729,4733,4751,4759,4783,4787,4789,4793,4799,4801,4813,4817,4831,4861,4871,4877,4889,4903,4909,4919,4931,4933,4937,4943,4951,4957,4967,4969,4973,4987,4993,4999,5003,5009,5011,5021,5023,5039,5051,5059,5077,5081,5087,5099,5101,5107,5113,5119,5147,5153,5167,5171,5179,5189,5197,5209,5227,5231,5233,5237,5261,5273,5279,5281,5297,5303,5309,5323,5333,5347,5351,5381,5387,5393,5399,5407,5413,5417,5419,5431,5437,5441,5443,5449,5471,5477,5479,5483,5501,5503,5507,5519,5521,5527,5531,5557,5563,5569,5573,5581,5591,5623,5639,5641,5647,5651,5653,5657,5659,5669,5683,5689,5693,5701,5711,5717,5737,5741,5743,5749,5779,5783,5791,5801,5807,5813,5821,5827,5839,5843,5849,5851,5857,5861,5867,5869,5879,5881,5897,5903,5923,5927,5939,5953,5981,5987,6007,6011,6029,6037,6043,6047,6053,6067,6073,6079,6089,6091,6101,6113,6121,6131,6133,6143,6151,6163,6173,6197,6199,6203,6211,6217,6221,6229,6247,6257,6263,6269,6271,6277,6287,6299,6301,6311,6317,6323,6329,6337,6343,6353,6359,6361,6367,6373,6379,6389,6397,6421,6427,6449,6451,6469,6473,6481,6491,6521,6529,6547,6551,6553,6563,6569,6571,6577,6581,6599,6607,6619,6637,6653,6659,6661,6673,6679,6689,6691,6701,6703,6709,6719,6733,6737,6761,6763,6779,6781,6791,6793,6803,6823,6827,6829,6833,6841,6857,6863,6869,6871,6883,6899,6907,6911,6917,6947,6949,6959,6961,6967,6971,6977,6983,6991,6997,7001,7013,7019,7027,7039,7043,7057,7069,7079,7103,7109,7121,7127,7129,7151,7159,7177,7187,7193,7207,7211,7213,7219,7229,7237,7243,7247,7253,7283,7297,7307,7309,7321,7331,7333,7349,7351,7369,7393,7411,7417,7433,7451,7457,7459,7477,7481,7487,7489,7499,7507,7517,7523,7529,7537,7541,7547,7549,7559,7561,7573,7577,7583,7589,7591,7603,7607,7621,7639,7643,7649,7669,7673,7681,7687,7691,7699,7703,7717,7723,7727,7741,7753,7757,7759,7789,7793,7817,7823,7829,7841,7853,7867,7873,7877,7879,7883,7901,7907,7919,7927,7933,7937,7949,7951,7963,7993,8009,8011,8017,8039,8053,8059,8069,8081,8087,8089,8093,8101,8111,8117,8123,8147,8161,8167,8171,8179,8191,8209,8219,8221,8231,8233,8237,8243,8263,8269,8273,8287,8291,8293,8297,8311,8317,8329,8353,8363,8369,8377,8387,8389,8419,8423,8429,8431,8443,8447,8461,8467,8501,8513,8521,8527,8537,8539,8543,8563,8573,8581,8597,8599,8609,8623,8627,8629,8641,8647,8663,8669,8677,8681,8689,8693,8699,8707,8713,8719,8731,8737,8741,8747,8753,8761,8779,8783,8803,8807,8819,8821,8831,8837,8839,8849,8861,8863,8867,8887,8893,8923,8929,8933,8941,8951,8963,8969,8971,8999,9001,9007,9011,9013,9029,9041,9043,9049,9059,9067,9091,9103,9109,9127,9133,9137,9151,9157,9161,9173,9181,9187,9199,9203,9209,9221,9227,9239,9241,9257,9277,9281,9283,9293,9311,9319,9323,9337,9341,9343,9349,9371,9377,9391,9397,9403,9413,9419,9421,9431,9433,9437,9439,9461,9463,9467,9473,9479,9491,9497,9511,9521,9533,9539,9547,9551,9587,9601,9613,9619,9623,9629,9631,9643,9649,9661,9677,9679,9689,9697,9719,9721,9733,9739,9743,9749,9767,9769,9781,9787,9791,9803,9811,9817,9829,9833,9839,9851,9857,9859,9871,9883,9887,9901,9907,9923,9929,9931,9941,9949,9967,9973,10007,10009,10037,10039,10061,10067,10069,10079,10091,10093,10099,10103,10111,10133,10139,10141,10151,10159,10163,10169,10177,10181,10193,10211,10223,10243,10247,10253,10259,10267,10271,10273,10289,10301,10303,10313,10321,10331,10333,10337,10343,10357,10369,10391,10399,10427,10429,10433,10453,10457,10459,10463,10477,10487,10499,10501,10513,10529,10531,10559,10567,10589,10597,10601,10607,10613,10627,10631,10639,10651,10657,10663,10667,10687,10691,10709,10711,10723,10729,10733,10739,10753,10771,10781,10789,10799,10831,10837,10847,10853,10859,10861,10867,10883,10889,10891,10903,10909,10937,10939,10949,10957,10973,10979,10987,10993,11003,11027,11047,11057,11059,11069,11071,11083,11087,11093,11113,11117,11119,11131,11149,11159,11161,11171,11173,11177,11197,11213,11239,11243,11251,11257,11261,11273,11279,11287,11299,11311,11317,11321,11329,11351,11353,11369,11383,11393,11399,11411,11423,11437,11443,11447,11467,11471,11483,11489,11491,11497,11503,11519,11527,11549,11551,11579,11587,11593,11597,11617,11621,11633,11657,11677,11681,11689,11699,11701,11717,11719,11731,11743,11777,11779,11783,11789,11801,11807,11813,11821,11827,11831,11833,11839,11863,11867,11887,11897,11903,11909,11923,11927,11933,11939,11941,11953,11959,11969,11971,11981,11987,12007,12011,12037,12041,12043,12049,12071,12073,12097,12101,12107,12109,12113,12119,12143,12149,12157,12161,12163,12197,12203,12211,12227,12239,12241,12251,12253,12263,12269,12277,12281,12289,12301,12323,12329,12343,12347,12373,12377,12379,12391,12401,12409,12413,12421,12433,12437,12451,12457,12473,12479,12487,12491,12497,12503,12511,12517,12527,12539,12541,12547,12553,12569,12577,12583,12589,12601,12611,12613,12619,12637,12641,12647,12653,12659,12671,12689,12697,12703,12713,12721,12739,12743,12757,12763,12781,12791,12799,12809,12821,12823,12829,12841,12853,12889,12893,12899,12907,12911,12917,12919,12923,12941,12953,12959,12967,12973,12979,12983,13001,13003,13007,13009,13033,13037,13043,13049,13063,13093,13099,13103,13109,13121,13127,13147,13151,13159,13163,13171,13177,13183,13187,13217,13219,13229,13241,13249,13259,13267,13291,13297,13309,13313,13327,13331,13337,13339,13367,13381,13397,13399,13411,13417,13421,13441,13451,13457,13463,13469,13477,13487,13499,13513,13523,13537,13553,13567,13577,13591,13597,13613,13619,13627,13633,13649,13669,13679,13681,13687,13691,13693,13697,13709,13711,13721,13723,13729,13751,13757,13759,13763,13781,13789,13799,13807,13829,13831,13841,13859,13873,13877,13879,13883,13901,13903,13907,13913,13921,13931,13933,13963,13967,13997,13999,14009,14011,14029,14033,14051,14057,14071,14081,14083,14087,14107,14143,14149,14153,14159,14173,14177,14197,14207,14221,14243,14249,14251,14281,14293,14303,14321,14323,14327,14341,14347,14369,14387,14389,14401,14407,14411,14419,14423,14431,14437,14447,14449,14461,14479,14489,14503,14519,14533,14537,14543,14549,14551,14557,14561,14563,14591,14593,14621,14627,14629,14633,14639,14653,14657,14669,14683,14699,14713,14717,14723,14731,14737,14741,14747,14753,14759,14767,14771,14779,14783,14797,14813,14821,14827,14831,14843,14851,14867,14869,14879,14887,14891,14897,14923,14929,14939,14947,14951,14957,14969,14983,15013,15017,15031,15053,15061,15073,15077,15083,15091,15101,15107,15121,15131,15137,15139,15149,15161,15173,15187,15193,15199,15217,15227,15233,15241,15259,15263,15269,15271,15277,15287,15289,15299,15307,15313,15319,15329,15331,15349,15359,15361,15373,15377,15383,15391,15401,15413,15427,15439,15443,15451,15461,15467,15473,15493,15497,15511,15527,15541,15551,15559,15569,15581,15583,15601,15607,15619,15629,15641,15643,15647,15649,15661,15667,15671,15679,15683,15727,15731,15733,15737,15739,15749,15761,15767,15773,15787,15791,15797,15803,15809,15817,15823,15859,15877,15881,15887,15889,15901,15907,15913,15919,15923,15937,15959,15971,15973,15991,16001,16007,16033,16057,16061,16063,16067,16069,16073,16087,16091,16097,16103,16111,16127,16139,16141,16183,16187,16189,16193,16217,16223,16229,16231,16249,16253,16267,16273,16301,16319,16333,16339,16349,16361,16363,16369,16381,16411,16417,16421,16427,16433,16447,16451,16453,16477,16481,16487,16493,16519,16529,16547,16553,16561,16567,16573,16603,16607,16619,16631,16633,16649,16651,16657,16661,16673,16691,16693,16699,16703,16729,16741,16747,16759,16763,16787,16811,16823,16829,16831,16843,16871,16879,16883,16889,16901,16903,16921,16927,16931,16937,16943,16963,16979,16981,16987,16993,17011,17021,17027,17029,17033,17041,17047,17053,17077,17093,17099,17107,17117,17123,17137,17159,17167,17183,17189,17191,17203,17207,17209,17231,17239,17257,17291,17293,17299,17317,17321,17327,17333,17341,17351,17359,17377,17383,17387,17389,17393,17401,17417,17419,17431,17443,17449,17467,17471,17477,17483,17489,17491,17497,17509,17519,17539,17551,17569,17573,17579,17581,17597,17599,17609,17623,17627,17657,17659,17669,17681,17683,17707,17713,17729,17737,17747,17749,17761,17783,17789,17791,17807,17827,17837,17839,17851,17863];

pub struct JpverbDspSIG0 {
	iJpverbDspSIG0Wave0_idx: i32,
}

impl JpverbDspSIG0 {
	
	fn get_num_inputsJpverbDspSIG0(&self) -> i32 {
		return 0;
	}
	fn get_num_outputsJpverbDspSIG0(&self) -> i32 {
		return 1;
	}
	
	pub fn instance_initJpverbDspSIG0(&mut self, sample_rate: i32) {
		self.iJpverbDspSIG0Wave0_idx = 0;
	}
	
	pub fn fillJpverbDspSIG0(&mut self, count: i32, table: &mut[i32]) {
		for i1 in 0..count {
			table[i1 as usize] = iJpverbDspSIG0Wave0[self.iJpverbDspSIG0Wave0_idx as usize];
			self.iJpverbDspSIG0Wave0_idx = (i32::wrapping_add(1, self.iJpverbDspSIG0Wave0_idx)) % 2048;
		}
	}

}


pub fn newJpverbDspSIG0() -> JpverbDspSIG0 { 
	JpverbDspSIG0 {
		iJpverbDspSIG0Wave0_idx: 0,
	}
}
fn JpverbDsp_faustpower2_f(value: F32) -> F32 {
	return value * value;
}
static itbl0JpverbDspSIG0: std::sync::RwLock<[i32;2048]>  = std::sync::RwLock::new([0;2048]);
fn remainder_f32(from: f32, to: f32) -> f32 {
	from - to * (from / to).round_ties_even()
}
fn rint_f32(val: f32) -> f32 {
	val.round_ties_even()
}

pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 10;
pub const FAUST_PASSIVES: usize = 0;


impl JpverbDsp {
		
	pub fn new() -> JpverbDsp { 
		JpverbDsp {
			fHslider0: 0.0,
			fHslider1: 0.0,
			fHslider2: 0.0,
			fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fHslider3: 0.0,
			iVec0: [0;2],
			fRec15: [0.0;2],
			fRec16: [0.0;2],
			fHslider4: 0.0,
			fHslider5: 0.0,
			fHslider6: 0.0,
			fConst2: 0.0,
			fHslider7: 0.0,
			IOTA0: 0,
			fVec1: [0.0;16384],
			fRec53: [0.0;2],
			fVec2: [0.0;2],
			fRec52: [0.0;2],
			fRec50: [0.0;2],
			fVec3: [0.0;16384],
			fRec55: [0.0;2],
			fVec4: [0.0;2],
			fRec54: [0.0;2],
			fRec51: [0.0;2],
			fVec5: [0.0;16384],
			fRec56: [0.0;2],
			fVec6: [0.0;2],
			fRec49: [0.0;2],
			fRec47: [0.0;2],
			fVec7: [0.0;16384],
			fRec58: [0.0;2],
			fVec8: [0.0;2],
			fRec57: [0.0;2],
			fRec48: [0.0;2],
			fVec9: [0.0;16384],
			fRec59: [0.0;2],
			fVec10: [0.0;2],
			fRec46: [0.0;2],
			fRec44: [0.0;2],
			fVec11: [0.0;16384],
			fRec61: [0.0;2],
			fVec12: [0.0;2],
			fRec60: [0.0;2],
			fRec45: [0.0;2],
			fVec13: [0.0;16384],
			fRec62: [0.0;2],
			fVec14: [0.0;2],
			fRec43: [0.0;2],
			fRec41: [0.0;2],
			fVec15: [0.0;16384],
			fRec64: [0.0;2],
			fVec16: [0.0;2],
			fRec63: [0.0;2],
			fRec42: [0.0;2],
			fVec17: [0.0;16384],
			fRec65: [0.0;2],
			fVec18: [0.0;2],
			fRec40: [0.0;2],
			fRec38: [0.0;2],
			fVec19: [0.0;16384],
			fRec67: [0.0;2],
			fVec20: [0.0;2],
			fRec66: [0.0;2],
			fRec39: [0.0;2],
			fVec21: [0.0;1024],
			fVec22: [0.0;16384],
			fRec68: [0.0;2],
			fVec23: [0.0;2],
			fRec37: [0.0;2],
			fVec24: [0.0;1024],
			fVec25: [0.0;16384],
			fRec70: [0.0;2],
			fVec26: [0.0;2],
			fRec69: [0.0;2],
			fVec27: [0.0;16384],
			fRec71: [0.0;2],
			fVec28: [0.0;2],
			fRec36: [0.0;2],
			fRec34: [0.0;2],
			fVec29: [0.0;16384],
			fRec73: [0.0;2],
			fVec30: [0.0;2],
			fRec72: [0.0;2],
			fRec35: [0.0;2],
			fVec31: [0.0;16384],
			fRec74: [0.0;2],
			fVec32: [0.0;2],
			fRec33: [0.0;2],
			fRec31: [0.0;2],
			fVec33: [0.0;16384],
			fRec76: [0.0;2],
			fVec34: [0.0;2],
			fRec75: [0.0;2],
			fRec32: [0.0;2],
			fVec35: [0.0;16384],
			fRec77: [0.0;2],
			fVec36: [0.0;2],
			fRec30: [0.0;2],
			fRec28: [0.0;2],
			fVec37: [0.0;16384],
			fRec79: [0.0;2],
			fVec38: [0.0;2],
			fRec78: [0.0;2],
			fRec29: [0.0;2],
			fVec39: [0.0;16384],
			fRec80: [0.0;2],
			fVec40: [0.0;2],
			fRec27: [0.0;2],
			fRec25: [0.0;2],
			fVec41: [0.0;16384],
			fRec82: [0.0;2],
			fVec42: [0.0;2],
			fRec81: [0.0;2],
			fRec26: [0.0;2],
			fVec43: [0.0;16384],
			fRec83: [0.0;2],
			fVec44: [0.0;2],
			fRec24: [0.0;2],
			fRec22: [0.0;2],
			fVec45: [0.0;16384],
			fRec85: [0.0;2],
			fVec46: [0.0;2],
			fRec84: [0.0;2],
			fRec23: [0.0;2],
			fVec47: [0.0;16384],
			fVec48: [0.0;16384],
			fRec86: [0.0;2],
			fVec49: [0.0;2],
			fRec21: [0.0;2],
			fRec20: [0.0;2],
			fRec19: [0.0;3],
			fRec18: [0.0;3],
			fHslider8: 0.0,
			fRec17: [0.0;3],
			fRec92: [0.0;2],
			fRec91: [0.0;3],
			fRec90: [0.0;3],
			fVec50: [0.0;2],
			fRec89: [0.0;2],
			fRec88: [0.0;3],
			fRec87: [0.0;3],
			fHslider9: 0.0,
			fRec95: [0.0;2],
			fRec94: [0.0;3],
			fRec93: [0.0;3],
			fVec51: [0.0;1024],
			fRec14: [0.0;2],
			fVec52: [0.0;16384],
			fVec53: [0.0;16384],
			fRec102: [0.0;2],
			fVec54: [0.0;2],
			fRec101: [0.0;2],
			fRec100: [0.0;2],
			fRec99: [0.0;3],
			fRec98: [0.0;3],
			fRec97: [0.0;3],
			fRec108: [0.0;2],
			fRec107: [0.0;3],
			fRec106: [0.0;3],
			fVec55: [0.0;2],
			fRec105: [0.0;2],
			fRec104: [0.0;3],
			fRec103: [0.0;3],
			fRec111: [0.0;2],
			fRec110: [0.0;3],
			fRec109: [0.0;3],
			fVec56: [0.0;1024],
			fRec96: [0.0;2],
			fVec57: [0.0;16384],
			fVec58: [0.0;2],
			fRec13: [0.0;2],
			fRec11: [0.0;2],
			fVec59: [0.0;16384],
			fRec113: [0.0;2],
			fVec60: [0.0;2],
			fRec112: [0.0;2],
			fRec12: [0.0;2],
			fVec61: [0.0;16384],
			fVec62: [0.0;2],
			fRec10: [0.0;2],
			fRec8: [0.0;2],
			fVec63: [0.0;16384],
			fVec64: [0.0;2],
			fRec114: [0.0;2],
			fRec9: [0.0;2],
			fVec65: [0.0;16384],
			fVec66: [0.0;2],
			fRec7: [0.0;2],
			fRec5: [0.0;2],
			fVec67: [0.0;16384],
			fRec116: [0.0;2],
			fVec68: [0.0;2],
			fRec115: [0.0;2],
			fRec6: [0.0;2],
			fVec69: [0.0;16384],
			fRec117: [0.0;2],
			fVec70: [0.0;2],
			fRec4: [0.0;2],
			fRec2: [0.0;2],
			fVec71: [0.0;16384],
			fVec72: [0.0;2],
			fRec118: [0.0;2],
			fRec3: [0.0;2],
			fRec0: [0.0;2],
			fRec1: [0.0;2],
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("analyzers.lib/name", r"Faust Analyzer Library");
		m.declare("analyzers.lib/version", r"1.2.0");
		m.declare("basics.lib/name", r"Faust Basic Element Library");
		m.declare("basics.lib/version", r"1.21.0");
		m.declare("compile_options", r"-lang rust -ct 1 -cn JpverbDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("delays.lib/fdelay1a:author", r"Julius O. Smith III");
		m.declare("delays.lib/fdelay4:author", r"Julius O. Smith III");
		m.declare("delays.lib/fdelayltv:author", r"Julius O. Smith III");
		m.declare("delays.lib/name", r"Faust Delay Library");
		m.declare("delays.lib/version", r"1.2.0");
		m.declare("filename", r"jpverb.dsp");
		m.declare("filters.lib/filterbank:author", r"Julius O. Smith III");
		m.declare("filters.lib/filterbank:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/filterbank:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/fir:author", r"Julius O. Smith III");
		m.declare("filters.lib/fir:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/fir:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/highpass:author", r"Julius O. Smith III");
		m.declare("filters.lib/highpass:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/highpass_plus_lowpass:author", r"Julius O. Smith III");
		m.declare("filters.lib/highpass_plus_lowpass:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/highpass_plus_lowpass:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/iir:author", r"Julius O. Smith III");
		m.declare("filters.lib/iir:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/iir:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/lowpass0_highpass1", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/lowpass0_highpass1:author", r"Julius O. Smith III");
		m.declare("filters.lib/lowpass:author", r"Julius O. Smith III");
		m.declare("filters.lib/lowpass:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/lowpass:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/name", r"Faust Filters Library");
		m.declare("filters.lib/nlf2:author", r"Julius O. Smith III");
		m.declare("filters.lib/nlf2:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/nlf2:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/tf1:author", r"Julius O. Smith III");
		m.declare("filters.lib/tf1:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/tf1:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/tf1s:author", r"Julius O. Smith III");
		m.declare("filters.lib/tf1s:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/tf1s:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/tf2:author", r"Julius O. Smith III");
		m.declare("filters.lib/tf2:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/tf2:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/tf2s:author", r"Julius O. Smith III");
		m.declare("filters.lib/tf2s:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/tf2s:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/version", r"1.7.1");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("name", r"jpverb");
		m.declare("oscillators.lib/name", r"Faust Oscillator Library");
		m.declare("oscillators.lib/version", r"1.6.0");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
		m.declare("reverbs.lib/jpverb:author", r"Julian Parker, bug fixes and minor interface changes by Till Bovermann");
		m.declare("reverbs.lib/jpverb:license", r"MIT license");
		m.declare("reverbs.lib/name", r"Faust Reverb Library");
		m.declare("reverbs.lib/version", r"1.4.0");
		m.declare("signals.lib/name", r"Faust Signal Routing Library");
		m.declare("signals.lib/version", r"1.6.0");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 1 static var(s)
		let mut itbl0JpverbDspSIG0_guard = itbl0JpverbDspSIG0.write().unwrap();
		let mut sig0: JpverbDspSIG0 = newJpverbDspSIG0();
		sig0.instance_initJpverbDspSIG0(sample_rate);
		sig0.fillJpverbDspSIG0(2048, itbl0JpverbDspSIG0_guard.as_mut());
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.6;
		self.fHslider1 = 0.7;
		self.fHslider2 = 0.3;
		self.fHslider3 = 0.65;
		self.fHslider4 = 0.75;
		self.fHslider5 = 0.55;
		self.fHslider6 = 0.5;
		self.fHslider7 = 0.7;
		self.fHslider8 = 0.5;
		self.fHslider9 = 0.4;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.iVec0[l0 as usize] = 0;
		}
		for l1 in 0..2 {
			self.fRec15[l1 as usize] = 0.0;
		}
		for l2 in 0..2 {
			self.fRec16[l2 as usize] = 0.0;
		}
		self.IOTA0 = 0;
		for l3 in 0..16384 {
			self.fVec1[l3 as usize] = 0.0;
		}
		for l4 in 0..2 {
			self.fRec53[l4 as usize] = 0.0;
		}
		for l5 in 0..2 {
			self.fVec2[l5 as usize] = 0.0;
		}
		for l6 in 0..2 {
			self.fRec52[l6 as usize] = 0.0;
		}
		for l7 in 0..2 {
			self.fRec50[l7 as usize] = 0.0;
		}
		for l8 in 0..16384 {
			self.fVec3[l8 as usize] = 0.0;
		}
		for l9 in 0..2 {
			self.fRec55[l9 as usize] = 0.0;
		}
		for l10 in 0..2 {
			self.fVec4[l10 as usize] = 0.0;
		}
		for l11 in 0..2 {
			self.fRec54[l11 as usize] = 0.0;
		}
		for l12 in 0..2 {
			self.fRec51[l12 as usize] = 0.0;
		}
		for l13 in 0..16384 {
			self.fVec5[l13 as usize] = 0.0;
		}
		for l14 in 0..2 {
			self.fRec56[l14 as usize] = 0.0;
		}
		for l15 in 0..2 {
			self.fVec6[l15 as usize] = 0.0;
		}
		for l16 in 0..2 {
			self.fRec49[l16 as usize] = 0.0;
		}
		for l17 in 0..2 {
			self.fRec47[l17 as usize] = 0.0;
		}
		for l18 in 0..16384 {
			self.fVec7[l18 as usize] = 0.0;
		}
		for l19 in 0..2 {
			self.fRec58[l19 as usize] = 0.0;
		}
		for l20 in 0..2 {
			self.fVec8[l20 as usize] = 0.0;
		}
		for l21 in 0..2 {
			self.fRec57[l21 as usize] = 0.0;
		}
		for l22 in 0..2 {
			self.fRec48[l22 as usize] = 0.0;
		}
		for l23 in 0..16384 {
			self.fVec9[l23 as usize] = 0.0;
		}
		for l24 in 0..2 {
			self.fRec59[l24 as usize] = 0.0;
		}
		for l25 in 0..2 {
			self.fVec10[l25 as usize] = 0.0;
		}
		for l26 in 0..2 {
			self.fRec46[l26 as usize] = 0.0;
		}
		for l27 in 0..2 {
			self.fRec44[l27 as usize] = 0.0;
		}
		for l28 in 0..16384 {
			self.fVec11[l28 as usize] = 0.0;
		}
		for l29 in 0..2 {
			self.fRec61[l29 as usize] = 0.0;
		}
		for l30 in 0..2 {
			self.fVec12[l30 as usize] = 0.0;
		}
		for l31 in 0..2 {
			self.fRec60[l31 as usize] = 0.0;
		}
		for l32 in 0..2 {
			self.fRec45[l32 as usize] = 0.0;
		}
		for l33 in 0..16384 {
			self.fVec13[l33 as usize] = 0.0;
		}
		for l34 in 0..2 {
			self.fRec62[l34 as usize] = 0.0;
		}
		for l35 in 0..2 {
			self.fVec14[l35 as usize] = 0.0;
		}
		for l36 in 0..2 {
			self.fRec43[l36 as usize] = 0.0;
		}
		for l37 in 0..2 {
			self.fRec41[l37 as usize] = 0.0;
		}
		for l38 in 0..16384 {
			self.fVec15[l38 as usize] = 0.0;
		}
		for l39 in 0..2 {
			self.fRec64[l39 as usize] = 0.0;
		}
		for l40 in 0..2 {
			self.fVec16[l40 as usize] = 0.0;
		}
		for l41 in 0..2 {
			self.fRec63[l41 as usize] = 0.0;
		}
		for l42 in 0..2 {
			self.fRec42[l42 as usize] = 0.0;
		}
		for l43 in 0..16384 {
			self.fVec17[l43 as usize] = 0.0;
		}
		for l44 in 0..2 {
			self.fRec65[l44 as usize] = 0.0;
		}
		for l45 in 0..2 {
			self.fVec18[l45 as usize] = 0.0;
		}
		for l46 in 0..2 {
			self.fRec40[l46 as usize] = 0.0;
		}
		for l47 in 0..2 {
			self.fRec38[l47 as usize] = 0.0;
		}
		for l48 in 0..16384 {
			self.fVec19[l48 as usize] = 0.0;
		}
		for l49 in 0..2 {
			self.fRec67[l49 as usize] = 0.0;
		}
		for l50 in 0..2 {
			self.fVec20[l50 as usize] = 0.0;
		}
		for l51 in 0..2 {
			self.fRec66[l51 as usize] = 0.0;
		}
		for l52 in 0..2 {
			self.fRec39[l52 as usize] = 0.0;
		}
		for l53 in 0..1024 {
			self.fVec21[l53 as usize] = 0.0;
		}
		for l54 in 0..16384 {
			self.fVec22[l54 as usize] = 0.0;
		}
		for l55 in 0..2 {
			self.fRec68[l55 as usize] = 0.0;
		}
		for l56 in 0..2 {
			self.fVec23[l56 as usize] = 0.0;
		}
		for l57 in 0..2 {
			self.fRec37[l57 as usize] = 0.0;
		}
		for l58 in 0..1024 {
			self.fVec24[l58 as usize] = 0.0;
		}
		for l59 in 0..16384 {
			self.fVec25[l59 as usize] = 0.0;
		}
		for l60 in 0..2 {
			self.fRec70[l60 as usize] = 0.0;
		}
		for l61 in 0..2 {
			self.fVec26[l61 as usize] = 0.0;
		}
		for l62 in 0..2 {
			self.fRec69[l62 as usize] = 0.0;
		}
		for l63 in 0..16384 {
			self.fVec27[l63 as usize] = 0.0;
		}
		for l64 in 0..2 {
			self.fRec71[l64 as usize] = 0.0;
		}
		for l65 in 0..2 {
			self.fVec28[l65 as usize] = 0.0;
		}
		for l66 in 0..2 {
			self.fRec36[l66 as usize] = 0.0;
		}
		for l67 in 0..2 {
			self.fRec34[l67 as usize] = 0.0;
		}
		for l68 in 0..16384 {
			self.fVec29[l68 as usize] = 0.0;
		}
		for l69 in 0..2 {
			self.fRec73[l69 as usize] = 0.0;
		}
		for l70 in 0..2 {
			self.fVec30[l70 as usize] = 0.0;
		}
		for l71 in 0..2 {
			self.fRec72[l71 as usize] = 0.0;
		}
		for l72 in 0..2 {
			self.fRec35[l72 as usize] = 0.0;
		}
		for l73 in 0..16384 {
			self.fVec31[l73 as usize] = 0.0;
		}
		for l74 in 0..2 {
			self.fRec74[l74 as usize] = 0.0;
		}
		for l75 in 0..2 {
			self.fVec32[l75 as usize] = 0.0;
		}
		for l76 in 0..2 {
			self.fRec33[l76 as usize] = 0.0;
		}
		for l77 in 0..2 {
			self.fRec31[l77 as usize] = 0.0;
		}
		for l78 in 0..16384 {
			self.fVec33[l78 as usize] = 0.0;
		}
		for l79 in 0..2 {
			self.fRec76[l79 as usize] = 0.0;
		}
		for l80 in 0..2 {
			self.fVec34[l80 as usize] = 0.0;
		}
		for l81 in 0..2 {
			self.fRec75[l81 as usize] = 0.0;
		}
		for l82 in 0..2 {
			self.fRec32[l82 as usize] = 0.0;
		}
		for l83 in 0..16384 {
			self.fVec35[l83 as usize] = 0.0;
		}
		for l84 in 0..2 {
			self.fRec77[l84 as usize] = 0.0;
		}
		for l85 in 0..2 {
			self.fVec36[l85 as usize] = 0.0;
		}
		for l86 in 0..2 {
			self.fRec30[l86 as usize] = 0.0;
		}
		for l87 in 0..2 {
			self.fRec28[l87 as usize] = 0.0;
		}
		for l88 in 0..16384 {
			self.fVec37[l88 as usize] = 0.0;
		}
		for l89 in 0..2 {
			self.fRec79[l89 as usize] = 0.0;
		}
		for l90 in 0..2 {
			self.fVec38[l90 as usize] = 0.0;
		}
		for l91 in 0..2 {
			self.fRec78[l91 as usize] = 0.0;
		}
		for l92 in 0..2 {
			self.fRec29[l92 as usize] = 0.0;
		}
		for l93 in 0..16384 {
			self.fVec39[l93 as usize] = 0.0;
		}
		for l94 in 0..2 {
			self.fRec80[l94 as usize] = 0.0;
		}
		for l95 in 0..2 {
			self.fVec40[l95 as usize] = 0.0;
		}
		for l96 in 0..2 {
			self.fRec27[l96 as usize] = 0.0;
		}
		for l97 in 0..2 {
			self.fRec25[l97 as usize] = 0.0;
		}
		for l98 in 0..16384 {
			self.fVec41[l98 as usize] = 0.0;
		}
		for l99 in 0..2 {
			self.fRec82[l99 as usize] = 0.0;
		}
		for l100 in 0..2 {
			self.fVec42[l100 as usize] = 0.0;
		}
		for l101 in 0..2 {
			self.fRec81[l101 as usize] = 0.0;
		}
		for l102 in 0..2 {
			self.fRec26[l102 as usize] = 0.0;
		}
		for l103 in 0..16384 {
			self.fVec43[l103 as usize] = 0.0;
		}
		for l104 in 0..2 {
			self.fRec83[l104 as usize] = 0.0;
		}
		for l105 in 0..2 {
			self.fVec44[l105 as usize] = 0.0;
		}
		for l106 in 0..2 {
			self.fRec24[l106 as usize] = 0.0;
		}
		for l107 in 0..2 {
			self.fRec22[l107 as usize] = 0.0;
		}
		for l108 in 0..16384 {
			self.fVec45[l108 as usize] = 0.0;
		}
		for l109 in 0..2 {
			self.fRec85[l109 as usize] = 0.0;
		}
		for l110 in 0..2 {
			self.fVec46[l110 as usize] = 0.0;
		}
		for l111 in 0..2 {
			self.fRec84[l111 as usize] = 0.0;
		}
		for l112 in 0..2 {
			self.fRec23[l112 as usize] = 0.0;
		}
		for l113 in 0..16384 {
			self.fVec47[l113 as usize] = 0.0;
		}
		for l114 in 0..16384 {
			self.fVec48[l114 as usize] = 0.0;
		}
		for l115 in 0..2 {
			self.fRec86[l115 as usize] = 0.0;
		}
		for l116 in 0..2 {
			self.fVec49[l116 as usize] = 0.0;
		}
		for l117 in 0..2 {
			self.fRec21[l117 as usize] = 0.0;
		}
		for l118 in 0..2 {
			self.fRec20[l118 as usize] = 0.0;
		}
		for l119 in 0..3 {
			self.fRec19[l119 as usize] = 0.0;
		}
		for l120 in 0..3 {
			self.fRec18[l120 as usize] = 0.0;
		}
		for l121 in 0..3 {
			self.fRec17[l121 as usize] = 0.0;
		}
		for l122 in 0..2 {
			self.fRec92[l122 as usize] = 0.0;
		}
		for l123 in 0..3 {
			self.fRec91[l123 as usize] = 0.0;
		}
		for l124 in 0..3 {
			self.fRec90[l124 as usize] = 0.0;
		}
		for l125 in 0..2 {
			self.fVec50[l125 as usize] = 0.0;
		}
		for l126 in 0..2 {
			self.fRec89[l126 as usize] = 0.0;
		}
		for l127 in 0..3 {
			self.fRec88[l127 as usize] = 0.0;
		}
		for l128 in 0..3 {
			self.fRec87[l128 as usize] = 0.0;
		}
		for l129 in 0..2 {
			self.fRec95[l129 as usize] = 0.0;
		}
		for l130 in 0..3 {
			self.fRec94[l130 as usize] = 0.0;
		}
		for l131 in 0..3 {
			self.fRec93[l131 as usize] = 0.0;
		}
		for l132 in 0..1024 {
			self.fVec51[l132 as usize] = 0.0;
		}
		for l133 in 0..2 {
			self.fRec14[l133 as usize] = 0.0;
		}
		for l134 in 0..16384 {
			self.fVec52[l134 as usize] = 0.0;
		}
		for l135 in 0..16384 {
			self.fVec53[l135 as usize] = 0.0;
		}
		for l136 in 0..2 {
			self.fRec102[l136 as usize] = 0.0;
		}
		for l137 in 0..2 {
			self.fVec54[l137 as usize] = 0.0;
		}
		for l138 in 0..2 {
			self.fRec101[l138 as usize] = 0.0;
		}
		for l139 in 0..2 {
			self.fRec100[l139 as usize] = 0.0;
		}
		for l140 in 0..3 {
			self.fRec99[l140 as usize] = 0.0;
		}
		for l141 in 0..3 {
			self.fRec98[l141 as usize] = 0.0;
		}
		for l142 in 0..3 {
			self.fRec97[l142 as usize] = 0.0;
		}
		for l143 in 0..2 {
			self.fRec108[l143 as usize] = 0.0;
		}
		for l144 in 0..3 {
			self.fRec107[l144 as usize] = 0.0;
		}
		for l145 in 0..3 {
			self.fRec106[l145 as usize] = 0.0;
		}
		for l146 in 0..2 {
			self.fVec55[l146 as usize] = 0.0;
		}
		for l147 in 0..2 {
			self.fRec105[l147 as usize] = 0.0;
		}
		for l148 in 0..3 {
			self.fRec104[l148 as usize] = 0.0;
		}
		for l149 in 0..3 {
			self.fRec103[l149 as usize] = 0.0;
		}
		for l150 in 0..2 {
			self.fRec111[l150 as usize] = 0.0;
		}
		for l151 in 0..3 {
			self.fRec110[l151 as usize] = 0.0;
		}
		for l152 in 0..3 {
			self.fRec109[l152 as usize] = 0.0;
		}
		for l153 in 0..1024 {
			self.fVec56[l153 as usize] = 0.0;
		}
		for l154 in 0..2 {
			self.fRec96[l154 as usize] = 0.0;
		}
		for l155 in 0..16384 {
			self.fVec57[l155 as usize] = 0.0;
		}
		for l156 in 0..2 {
			self.fVec58[l156 as usize] = 0.0;
		}
		for l157 in 0..2 {
			self.fRec13[l157 as usize] = 0.0;
		}
		for l158 in 0..2 {
			self.fRec11[l158 as usize] = 0.0;
		}
		for l159 in 0..16384 {
			self.fVec59[l159 as usize] = 0.0;
		}
		for l160 in 0..2 {
			self.fRec113[l160 as usize] = 0.0;
		}
		for l161 in 0..2 {
			self.fVec60[l161 as usize] = 0.0;
		}
		for l162 in 0..2 {
			self.fRec112[l162 as usize] = 0.0;
		}
		for l163 in 0..2 {
			self.fRec12[l163 as usize] = 0.0;
		}
		for l164 in 0..16384 {
			self.fVec61[l164 as usize] = 0.0;
		}
		for l165 in 0..2 {
			self.fVec62[l165 as usize] = 0.0;
		}
		for l166 in 0..2 {
			self.fRec10[l166 as usize] = 0.0;
		}
		for l167 in 0..2 {
			self.fRec8[l167 as usize] = 0.0;
		}
		for l168 in 0..16384 {
			self.fVec63[l168 as usize] = 0.0;
		}
		for l169 in 0..2 {
			self.fVec64[l169 as usize] = 0.0;
		}
		for l170 in 0..2 {
			self.fRec114[l170 as usize] = 0.0;
		}
		for l171 in 0..2 {
			self.fRec9[l171 as usize] = 0.0;
		}
		for l172 in 0..16384 {
			self.fVec65[l172 as usize] = 0.0;
		}
		for l173 in 0..2 {
			self.fVec66[l173 as usize] = 0.0;
		}
		for l174 in 0..2 {
			self.fRec7[l174 as usize] = 0.0;
		}
		for l175 in 0..2 {
			self.fRec5[l175 as usize] = 0.0;
		}
		for l176 in 0..16384 {
			self.fVec67[l176 as usize] = 0.0;
		}
		for l177 in 0..2 {
			self.fRec116[l177 as usize] = 0.0;
		}
		for l178 in 0..2 {
			self.fVec68[l178 as usize] = 0.0;
		}
		for l179 in 0..2 {
			self.fRec115[l179 as usize] = 0.0;
		}
		for l180 in 0..2 {
			self.fRec6[l180 as usize] = 0.0;
		}
		for l181 in 0..16384 {
			self.fVec69[l181 as usize] = 0.0;
		}
		for l182 in 0..2 {
			self.fRec117[l182 as usize] = 0.0;
		}
		for l183 in 0..2 {
			self.fVec70[l183 as usize] = 0.0;
		}
		for l184 in 0..2 {
			self.fRec4[l184 as usize] = 0.0;
		}
		for l185 in 0..2 {
			self.fRec2[l185 as usize] = 0.0;
		}
		for l186 in 0..16384 {
			self.fVec71[l186 as usize] = 0.0;
		}
		for l187 in 0..2 {
			self.fVec72[l187 as usize] = 0.0;
		}
		for l188 in 0..2 {
			self.fRec118[l188 as usize] = 0.0;
		}
		for l189 in 0..2 {
			self.fRec3[l189 as usize] = 0.0;
		}
		for l190 in 0..2 {
			self.fRec0[l190 as usize] = 0.0;
		}
		for l191 in 0..2 {
			self.fRec1[l191 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 1 static var(s)
		let itbl0JpverbDspSIG0_guard = itbl0JpverbDspSIG0.read().unwrap();
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = 62.831852 / self.fConst0;
		self.fConst2 = 3.1415927 / self.fConst0;
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		JpverbDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("jpverb");
		ui_interface.add_horizontal_slider("a_decay", ParamIndex(0), 0.55, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("b_damp", ParamIndex(1), 0.7, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("c_size", ParamIndex(2), 0.75, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("d_diff", ParamIndex(3), 0.6, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("e_moddepth", ParamIndex(4), 0.3, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("f_modfreq", ParamIndex(5), 0.65, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("g_low", ParamIndex(6), 0.4, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("h_high", ParamIndex(7), 0.5, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("i_lowcut", ParamIndex(8), 0.5, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("j_highcut", ParamIndex(9), 0.7, 0.0, 1.0, 0.001);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			3 => Some(self.fHslider0),
			1 => Some(self.fHslider1),
			4 => Some(self.fHslider2),
			5 => Some(self.fHslider3),
			2 => Some(self.fHslider4),
			0 => Some(self.fHslider5),
			7 => Some(self.fHslider6),
			9 => Some(self.fHslider7),
			8 => Some(self.fHslider8),
			6 => Some(self.fHslider9),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			3 => { self.fHslider0 = value }
			1 => { self.fHslider1 = value }
			4 => { self.fHslider2 = value }
			5 => { self.fHslider3 = value }
			2 => { self.fHslider4 = value }
			0 => { self.fHslider5 = value }
			7 => { self.fHslider6 = value }
			9 => { self.fHslider7 = value }
			8 => { self.fHslider8 = value }
			6 => { self.fHslider9 = value }
			_ => {}
		}
	}
	
	pub fn compute(
		&mut self,
		count: usize,
		inputs: &[impl AsRef<[FaustFloat]>],
		outputs: &mut[impl AsMut<[FaustFloat]>],
	) {
		
		// Obtaining locks on 1 static var(s)
		let itbl0JpverbDspSIG0_guard = itbl0JpverbDspSIG0.read().unwrap();
		let [inputs0, inputs1, .. ] = inputs.as_ref() else { panic!("wrong number of input buffers"); };
		let inputs0 = inputs0.as_ref()[..count].iter();
		let inputs1 = inputs1.as_ref()[..count].iter();
		let [outputs0, outputs1, .. ] = outputs.as_mut() else { panic!("wrong number of output buffers"); };
		let outputs0 = outputs0.as_mut()[..count].iter_mut();
		let outputs1 = outputs1.as_mut()[..count].iter_mut();
		let mut fSlow0: F32 = self.fHslider0;
		let mut fSlow1: F32 = F32::cos(fSlow0);
		let mut fSlow2: F32 = self.fHslider1;
		let mut fSlow3: F32 = 1.0 - fSlow2;
		let mut fSlow4: F32 = 5e+01 * self.fHslider2;
		let mut fSlow5: F32 = self.fConst1 * self.fHslider3;
		let mut fSlow6: F32 = F32::sin(fSlow5);
		let mut fSlow7: F32 = F32::cos(fSlow5);
		let mut fSlow8: F32 = 4.5 * self.fHslider4 + 0.5;
		let mut fSlow9: F32 = F32::powf(1e+01, -(5.1 * ((1.25 * fSlow8 + -0.25) / F32::powf(2e+02, self.fHslider5))));
		let mut fSlow10: F32 = self.fHslider6;
		let mut fSlow11: F32 = F32::tan(self.fConst2 * (9e+03 * self.fHslider7 + 1e+03));
		let mut fSlow12: F32 = JpverbDsp_faustpower2_f(fSlow11);
		let mut fSlow13: F32 = 1.0 / fSlow11;
		let mut fSlow14: F32 = (fSlow13 + 0.618034) / fSlow11 + 1.0;
		let mut fSlow15: F32 = 1.0 / (fSlow12 * fSlow14);
		let mut fSlow16: F32 = (fSlow13 + 1.618034) / fSlow11 + 1.0;
		let mut fSlow17: F32 = 1.0 / (fSlow12 * fSlow16);
		let mut fSlow18: F32 = 1.0 / (fSlow13 + 1.0);
		let mut fSlow19: F32 = 1.0 - fSlow13;
		let mut iSlow20: i32 = itbl0JpverbDspSIG0_guard[((1e+01 * fSlow8) as i32) as usize];
		let mut fSlow21: F32 = 0.0001 * (iSlow20) as F32;
		let mut iSlow22: i32 = itbl0JpverbDspSIG0_guard[((1.1e+02 * fSlow8) as i32) as usize];
		let mut fSlow23: F32 = 0.0001 * (iSlow22) as F32;
		let mut iSlow24: i32 = itbl0JpverbDspSIG0_guard[((4e+01 * fSlow8) as i32) as usize];
		let mut fSlow25: F32 = 0.0001 * (iSlow24) as F32;
		let mut iSlow26: i32 = itbl0JpverbDspSIG0_guard[((1.4e+02 * fSlow8) as i32) as usize];
		let mut fSlow27: F32 = 0.0001 * (iSlow26) as F32;
		let mut iSlow28: i32 = itbl0JpverbDspSIG0_guard[((7e+01 * fSlow8) as i32) as usize];
		let mut fSlow29: F32 = 0.0001 * (iSlow28) as F32;
		let mut iSlow30: i32 = itbl0JpverbDspSIG0_guard[((1.7e+02 * fSlow8) as i32) as usize];
		let mut fSlow31: F32 = 0.0001 * (iSlow30) as F32;
		let mut iSlow32: i32 = itbl0JpverbDspSIG0_guard[((1e+02 * fSlow8) as i32) as usize];
		let mut fSlow33: F32 = 0.0001 * (iSlow32) as F32;
		let mut iSlow34: i32 = itbl0JpverbDspSIG0_guard[((2e+02 * fSlow8) as i32) as usize];
		let mut fSlow35: F32 = 0.0001 * (iSlow34) as F32;
		let mut iSlow36: i32 = itbl0JpverbDspSIG0_guard[((1.3e+02 * fSlow8) as i32) as usize];
		let mut fSlow37: F32 = 0.0001 * (iSlow36) as F32;
		let mut iSlow38: i32 = itbl0JpverbDspSIG0_guard[((2.3e+02 * fSlow8) as i32) as usize];
		let mut fSlow39: F32 = 0.0001 * (iSlow38) as F32;
		let mut iSlow40: i32 = itbl0JpverbDspSIG0_guard[((54.0 * fSlow8) as i32) as usize];
		let mut fSlow41: F32 = 0.005 * (iSlow40) as F32;
		let mut iSlow42: i32 = itbl0JpverbDspSIG0_guard[((204.0 * fSlow8) as i32) as usize];
		let mut fSlow43: F32 = 0.005 * (iSlow42) as F32;
		let mut iSlow44: i32 = itbl0JpverbDspSIG0_guard[((125.0 * fSlow8) as i32) as usize];
		let mut fSlow45: F32 = 0.0001 * (iSlow44) as F32;
		let mut iSlow46: i32 = itbl0JpverbDspSIG0_guard[((25.0 * fSlow8) as i32) as usize];
		let mut fSlow47: F32 = 0.0001 * (iSlow46) as F32;
		let mut iSlow48: i32 = itbl0JpverbDspSIG0_guard[((155.0 * fSlow8) as i32) as usize];
		let mut fSlow49: F32 = 0.0001 * (iSlow48) as F32;
		let mut iSlow50: i32 = itbl0JpverbDspSIG0_guard[((55.0 * fSlow8) as i32) as usize];
		let mut fSlow51: F32 = 0.0001 * (iSlow50) as F32;
		let mut iSlow52: i32 = itbl0JpverbDspSIG0_guard[((185.0 * fSlow8) as i32) as usize];
		let mut fSlow53: F32 = 0.0001 * (iSlow52) as F32;
		let mut iSlow54: i32 = itbl0JpverbDspSIG0_guard[((85.0 * fSlow8) as i32) as usize];
		let mut fSlow55: F32 = 0.0001 * (iSlow54) as F32;
		let mut iSlow56: i32 = itbl0JpverbDspSIG0_guard[((215.0 * fSlow8) as i32) as usize];
		let mut fSlow57: F32 = 0.0001 * (iSlow56) as F32;
		let mut iSlow58: i32 = itbl0JpverbDspSIG0_guard[((115.0 * fSlow8) as i32) as usize];
		let mut fSlow59: F32 = 0.0001 * (iSlow58) as F32;
		let mut iSlow60: i32 = itbl0JpverbDspSIG0_guard[((245.0 * fSlow8) as i32) as usize];
		let mut fSlow61: F32 = 0.0001 * (iSlow60) as F32;
		let mut iSlow62: i32 = itbl0JpverbDspSIG0_guard[((145.0 * fSlow8) as i32) as usize];
		let mut fSlow63: F32 = 0.0001 * (iSlow62) as F32;
		let mut iSlow64: i32 = itbl0JpverbDspSIG0_guard[((134.0 * fSlow8) as i32) as usize];
		let mut fSlow65: F32 = 0.005 * (iSlow64) as F32;
		let mut fSlow66: F32 = 1.0 / fSlow16;
		let mut fSlow67: F32 = (fSlow13 + -1.618034) / fSlow11 + 1.0;
		let mut fSlow68: F32 = 2.0 * (1.0 - 1.0 / fSlow12);
		let mut fSlow69: F32 = 1.0 / fSlow14;
		let mut fSlow70: F32 = (fSlow13 + -0.618034) / fSlow11 + 1.0;
		let mut fSlow71: F32 = F32::tan(self.fConst2 * (5.9e+03 * self.fHslider8 + 1e+02));
		let mut fSlow72: F32 = 1.0 / fSlow71;
		let mut fSlow73: F32 = 1.0 / ((fSlow72 + 1.618034) / fSlow71 + 1.0);
		let mut fSlow74: F32 = (fSlow72 + -1.618034) / fSlow71 + 1.0;
		let mut fSlow75: F32 = JpverbDsp_faustpower2_f(fSlow71);
		let mut fSlow76: F32 = 1.0 / fSlow75;
		let mut fSlow77: F32 = 2.0 * (1.0 - fSlow76);
		let mut fSlow78: F32 = 1.0 / ((fSlow72 + 0.618034) / fSlow71 + 1.0);
		let mut fSlow79: F32 = (fSlow72 + 1.618034) / fSlow71 + 1.0;
		let mut fSlow80: F32 = 1.0 / (fSlow75 * fSlow79);
		let mut fSlow81: F32 = 1.0 / (fSlow72 + 1.0);
		let mut fSlow82: F32 = 1.0 - fSlow72;
		let mut fSlow83: F32 = 1.0 / fSlow79;
		let mut fSlow84: F32 = (fSlow72 + -1.618034) / fSlow71 + 1.0;
		let mut fSlow85: F32 = (fSlow72 + -0.618034) / fSlow71 + 1.0;
		let mut fSlow86: F32 = self.fHslider9;
		let mut fSlow87: F32 = F32::sin(fSlow0);
		let mut iSlow88: i32 = itbl0JpverbDspSIG0_guard[((34.0 * fSlow8) as i32) as usize];
		let mut fSlow89: F32 = 0.005 * (iSlow88) as F32;
		let mut iSlow90: i32 = itbl0JpverbDspSIG0_guard[((2.4e+02 * fSlow8) as i32) as usize];
		let mut fSlow91: F32 = 0.0001 * (iSlow90) as F32;
		let mut iSlow92: i32 = itbl0JpverbDspSIG0_guard[((1.9e+02 * fSlow8) as i32) as usize];
		let mut fSlow93: F32 = 0.0001 * (iSlow92) as F32;
		let mut iSlow94: i32 = itbl0JpverbDspSIG0_guard[((175.0 * fSlow8) as i32) as usize];
		let mut fSlow95: F32 = 0.0001 * (iSlow94) as F32;
		let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
		for (((input0, input1), output0), output1) in zipped_iterators {
			self.iVec0[0] = 1;
			self.fRec15[0] = fSlow6 * self.fRec16[1] + fSlow7 * self.fRec15[1];
			let mut iTemp0: i32 = i32::wrapping_sub(1, self.iVec0[1]);
			self.fRec16[0] = (iTemp0) as F32 + fSlow7 * self.fRec16[1] - fSlow6 * self.fRec15[1];
			let mut fTemp1: F32 = fSlow4 * (self.fRec15[0] + 1.0);
			let mut fTemp2: F32 = fTemp1 + 3.500005;
			let mut fTemp3: F32 = F32::floor(fTemp2);
			let mut fTemp4: F32 = fTemp1 + (1.0 - fTemp3);
			let mut fTemp5: F32 = fTemp1 + (2.0 - fTemp3);
			let mut fTemp6: F32 = fTemp1 + (3.0 - fTemp3);
			let mut fTemp7: F32 = fTemp1 + (4.0 - fTemp3);
			let mut fTemp8: F32 = fSlow4 * (self.fRec16[0] + 1.0);
			let mut fTemp9: F32 = fTemp8 + 3.500005;
			let mut fTemp10: F32 = F32::floor(fTemp9);
			let mut fTemp11: F32 = fTemp8 + (1.0 - fTemp10);
			let mut fTemp12: F32 = fTemp8 + (2.0 - fTemp10);
			let mut fTemp13: F32 = fTemp8 + (3.0 - fTemp10);
			let mut fTemp14: F32 = 0.7602446 * self.fRec0[1] - 0.6496369 * self.fRec50[1];
			let mut fTemp15: F32 = 0.6496369 * self.fRec51[1];
			let mut fTemp16: F32 = 0.7602446 * self.fRec1[1];
			self.fVec1[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp14 + 0.70710677 * (fTemp15 - fTemp16);
			self.fRec53[0] = 0.9999 * (self.fRec53[1] + (i32::wrapping_mul(iTemp0, iSlow20)) as F32) + fSlow21;
			let mut fTemp17: F32 = self.fRec53[0] + -1.49999;
			let mut fTemp18: F32 = self.fVec1[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp17) as i32)))) & 16383) as usize];
			self.fVec2[0] = fTemp18;
			let mut fTemp19: F32 = F32::floor(fTemp17);
			self.fRec52[0] = self.fVec2[1] - (fTemp19 + (2.0 - self.fRec53[0])) * (self.fRec52[1] - fTemp18) / (self.fRec53[0] - fTemp19);
			self.fRec50[0] = self.fRec52[0];
			self.fVec3[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp14 + 0.70710677 * (fTemp16 - fTemp15);
			self.fRec55[0] = 0.9999 * (self.fRec55[1] + (i32::wrapping_mul(iTemp0, iSlow22)) as F32) + fSlow23;
			let mut fTemp20: F32 = self.fRec55[0] + -1.49999;
			let mut fTemp21: F32 = self.fVec3[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp20) as i32)))) & 16383) as usize];
			self.fVec4[0] = fTemp21;
			let mut fTemp22: F32 = F32::floor(fTemp20);
			self.fRec54[0] = self.fVec4[1] - (fTemp22 + (2.0 - self.fRec55[0])) * (self.fRec54[1] - fTemp21) / (self.fRec55[0] - fTemp22);
			self.fRec51[0] = self.fRec54[0];
			let mut fTemp23: F32 = 0.7602446 * self.fRec50[1] + 0.6496369 * self.fRec0[1];
			let mut fTemp24: F32 = 0.7602446 * fTemp23 - 0.6496369 * self.fRec47[1];
			let mut fTemp25: F32 = 0.6496369 * self.fRec48[1];
			let mut fTemp26: F32 = 0.7602446 * self.fRec51[1] + 0.6496369 * self.fRec1[1];
			let mut fTemp27: F32 = 0.7602446 * fTemp26;
			self.fVec5[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp24 + 0.70710677 * (fTemp25 - fTemp27);
			self.fRec56[0] = 0.9999 * (self.fRec56[1] + (i32::wrapping_mul(iTemp0, iSlow24)) as F32) + fSlow25;
			let mut fTemp28: F32 = self.fRec56[0] + -1.49999;
			let mut fTemp29: F32 = self.fVec5[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp28) as i32)))) & 16383) as usize];
			self.fVec6[0] = fTemp29;
			let mut fTemp30: F32 = F32::floor(fTemp28);
			self.fRec49[0] = self.fVec6[1] - (fTemp30 + (2.0 - self.fRec56[0])) * (self.fRec49[1] - fTemp29) / (self.fRec56[0] - fTemp30);
			self.fRec47[0] = self.fRec49[0];
			self.fVec7[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp24 + 0.70710677 * (fTemp27 - fTemp25);
			self.fRec58[0] = 0.9999 * (self.fRec58[1] + (i32::wrapping_mul(iTemp0, iSlow26)) as F32) + fSlow27;
			let mut fTemp31: F32 = self.fRec58[0] + -1.49999;
			let mut fTemp32: F32 = self.fVec7[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp31) as i32)))) & 16383) as usize];
			self.fVec8[0] = fTemp32;
			let mut fTemp33: F32 = F32::floor(fTemp31);
			self.fRec57[0] = self.fVec8[1] - (fTemp33 + (2.0 - self.fRec58[0])) * (self.fRec57[1] - fTemp32) / (self.fRec58[0] - fTemp33);
			self.fRec48[0] = self.fRec57[0];
			let mut fTemp34: F32 = 0.7602446 * self.fRec47[1] + 0.6496369 * fTemp23;
			let mut fTemp35: F32 = 0.7602446 * fTemp34 - 0.6496369 * self.fRec44[1];
			let mut fTemp36: F32 = 0.6496369 * self.fRec45[1];
			let mut fTemp37: F32 = 0.7602446 * self.fRec48[1] + 0.6496369 * fTemp26;
			let mut fTemp38: F32 = 0.7602446 * fTemp37;
			self.fVec9[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp35 + 0.70710677 * (fTemp36 - fTemp38);
			self.fRec59[0] = 0.9999 * (self.fRec59[1] + (i32::wrapping_mul(iTemp0, iSlow28)) as F32) + fSlow29;
			let mut fTemp39: F32 = self.fRec59[0] + -1.49999;
			let mut fTemp40: F32 = self.fVec9[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp39) as i32)))) & 16383) as usize];
			self.fVec10[0] = fTemp40;
			let mut fTemp41: F32 = F32::floor(fTemp39);
			self.fRec46[0] = self.fVec10[1] - (fTemp41 + (2.0 - self.fRec59[0])) * (self.fRec46[1] - fTemp40) / (self.fRec59[0] - fTemp41);
			self.fRec44[0] = self.fRec46[0];
			self.fVec11[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp35 + 0.70710677 * (fTemp38 - fTemp36);
			self.fRec61[0] = 0.9999 * (self.fRec61[1] + (i32::wrapping_mul(iTemp0, iSlow30)) as F32) + fSlow31;
			let mut fTemp42: F32 = self.fRec61[0] + -1.49999;
			let mut fTemp43: F32 = self.fVec11[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp42) as i32)))) & 16383) as usize];
			self.fVec12[0] = fTemp43;
			let mut fTemp44: F32 = F32::floor(fTemp42);
			self.fRec60[0] = self.fVec12[1] - (fTemp44 + (2.0 - self.fRec61[0])) * (self.fRec60[1] - fTemp43) / (self.fRec61[0] - fTemp44);
			self.fRec45[0] = self.fRec60[0];
			let mut fTemp45: F32 = 0.7602446 * self.fRec44[1] + 0.6496369 * fTemp34;
			let mut fTemp46: F32 = 0.7602446 * fTemp45 - 0.6496369 * self.fRec41[1];
			let mut fTemp47: F32 = 0.6496369 * self.fRec42[1];
			let mut fTemp48: F32 = 0.7602446 * self.fRec45[1] + 0.6496369 * fTemp37;
			let mut fTemp49: F32 = 0.7602446 * fTemp48;
			self.fVec13[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp46 + 0.70710677 * (fTemp47 - fTemp49);
			self.fRec62[0] = 0.9999 * (self.fRec62[1] + (i32::wrapping_mul(iTemp0, iSlow32)) as F32) + fSlow33;
			let mut fTemp50: F32 = self.fRec62[0] + -1.49999;
			let mut fTemp51: F32 = self.fVec13[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp50) as i32)))) & 16383) as usize];
			self.fVec14[0] = fTemp51;
			let mut fTemp52: F32 = F32::floor(fTemp50);
			self.fRec43[0] = self.fVec14[1] - (fTemp52 + (2.0 - self.fRec62[0])) * (self.fRec43[1] - fTemp51) / (self.fRec62[0] - fTemp52);
			self.fRec41[0] = self.fRec43[0];
			self.fVec15[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp46 + 0.70710677 * (fTemp49 - fTemp47);
			self.fRec64[0] = 0.9999 * (self.fRec64[1] + (i32::wrapping_mul(iTemp0, iSlow34)) as F32) + fSlow35;
			let mut fTemp53: F32 = self.fRec64[0] + -1.49999;
			let mut fTemp54: F32 = self.fVec15[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp53) as i32)))) & 16383) as usize];
			self.fVec16[0] = fTemp54;
			let mut fTemp55: F32 = F32::floor(fTemp53);
			self.fRec63[0] = self.fVec16[1] - (fTemp55 + (2.0 - self.fRec64[0])) * (self.fRec63[1] - fTemp54) / (self.fRec64[0] - fTemp55);
			self.fRec42[0] = self.fRec63[0];
			let mut fTemp56: F32 = 0.7602446 * self.fRec41[1] + 0.6496369 * fTemp45;
			let mut fTemp57: F32 = 0.7602446 * fTemp56 - 0.6496369 * self.fRec38[1];
			let mut fTemp58: F32 = 0.6496369 * self.fRec39[1];
			let mut fTemp59: F32 = 0.7602446 * self.fRec42[1] + 0.6496369 * fTemp48;
			let mut fTemp60: F32 = 0.7602446 * fTemp59;
			self.fVec17[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp57 + 0.70710677 * (fTemp58 - fTemp60);
			self.fRec65[0] = 0.9999 * (self.fRec65[1] + (i32::wrapping_mul(iTemp0, iSlow36)) as F32) + fSlow37;
			let mut fTemp61: F32 = self.fRec65[0] + -1.49999;
			let mut fTemp62: F32 = self.fVec17[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp61) as i32)))) & 16383) as usize];
			self.fVec18[0] = fTemp62;
			let mut fTemp63: F32 = F32::floor(fTemp61);
			self.fRec40[0] = self.fVec18[1] - (fTemp63 + (2.0 - self.fRec65[0])) * (self.fRec40[1] - fTemp62) / (self.fRec65[0] - fTemp63);
			self.fRec38[0] = self.fRec40[0];
			self.fVec19[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp57 + 0.70710677 * (fTemp60 - fTemp58);
			self.fRec67[0] = 0.9999 * (self.fRec67[1] + (i32::wrapping_mul(iTemp0, iSlow38)) as F32) + fSlow39;
			let mut fTemp64: F32 = self.fRec67[0] + -1.49999;
			let mut fTemp65: F32 = self.fVec19[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp64) as i32)))) & 16383) as usize];
			self.fVec20[0] = fTemp65;
			let mut fTemp66: F32 = F32::floor(fTemp64);
			self.fRec66[0] = self.fVec20[1] - (fTemp66 + (2.0 - self.fRec67[0])) * (self.fRec66[1] - fTemp65) / (self.fRec67[0] - fTemp66);
			self.fRec39[0] = self.fRec66[0];
			let mut fTemp67: F32 = 0.7602446 * self.fRec38[1] + 0.6496369 * fTemp56;
			self.fVec21[(self.IOTA0 & 1023) as usize] = fTemp67;
			let mut iTemp68: i32 = (fTemp9) as i32;
			let mut iTemp69: i32 = std::cmp::min(512, std::cmp::max(0, iTemp68));
			let mut fTemp70: F32 = fTemp8 + (4.0 - fTemp10);
			let mut fTemp71: F32 = fTemp8 + (5.0 - fTemp10);
			let mut iTemp72: i32 = std::cmp::min(512, std::cmp::max(0, i32::wrapping_add(iTemp68, 1)));
			let mut fTemp73: F32 = fTemp71 * fTemp70;
			let mut iTemp74: i32 = std::cmp::min(512, std::cmp::max(0, i32::wrapping_add(iTemp68, 2)));
			let mut fTemp75: F32 = fTemp73 * fTemp13;
			let mut iTemp76: i32 = std::cmp::min(512, std::cmp::max(0, i32::wrapping_add(iTemp68, 3)));
			let mut fTemp77: F32 = fTemp75 * fTemp12;
			let mut iTemp78: i32 = std::cmp::min(512, std::cmp::max(0, i32::wrapping_add(iTemp68, 4)));
			self.fVec22[(self.IOTA0 & 16383) as usize] = fTemp11 * (fTemp12 * (fTemp13 * (0.041666668 * self.fVec21[((i32::wrapping_sub(self.IOTA0, iTemp69)) & 1023) as usize] * fTemp70 - 0.16666667 * fTemp71 * self.fVec21[((i32::wrapping_sub(self.IOTA0, iTemp72)) & 1023) as usize]) + 0.25 * fTemp73 * self.fVec21[((i32::wrapping_sub(self.IOTA0, iTemp74)) & 1023) as usize]) - 0.16666667 * fTemp75 * self.fVec21[((i32::wrapping_sub(self.IOTA0, iTemp76)) & 1023) as usize]) + 0.041666668 * fTemp77 * self.fVec21[((i32::wrapping_sub(self.IOTA0, iTemp78)) & 1023) as usize];
			self.fRec68[0] = 0.995 * (self.fRec68[1] + (i32::wrapping_mul(iTemp0, iSlow40)) as F32) + fSlow41;
			let mut fTemp79: F32 = self.fRec68[0] + -1.49999;
			let mut fTemp80: F32 = self.fVec22[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp79) as i32)))) & 16383) as usize];
			self.fVec23[0] = fTemp80;
			let mut fTemp81: F32 = F32::floor(fTemp79);
			self.fRec37[0] = self.fVec23[1] - (fTemp81 + (2.0 - self.fRec68[0])) * (self.fRec37[1] - fTemp80) / (self.fRec68[0] - fTemp81);
			let mut fTemp82: F32 = 0.7602446 * self.fRec37[0] - 0.6496369 * self.fRec34[1];
			let mut fTemp83: F32 = 0.6496369 * self.fRec35[1];
			let mut fTemp84: F32 = fSlow4 * (1.0 - self.fRec16[0]);
			let mut fTemp85: F32 = fTemp84 + 3.500005;
			let mut fTemp86: F32 = F32::floor(fTemp85);
			let mut fTemp87: F32 = fTemp84 + (2.0 - fTemp86);
			let mut fTemp88: F32 = fTemp84 + (3.0 - fTemp86);
			let mut fTemp89: F32 = 0.7602446 * self.fRec39[1] + 0.6496369 * fTemp59;
			self.fVec24[(self.IOTA0 & 1023) as usize] = fTemp89;
			let mut iTemp90: i32 = (fTemp85) as i32;
			let mut fTemp91: F32 = fTemp84 + (4.0 - fTemp86);
			let mut fTemp92: F32 = fTemp84 + (5.0 - fTemp86);
			let mut fTemp93: F32 = fTemp92 * fTemp91;
			let mut fTemp94: F32 = fTemp93 * fTemp88;
			self.fVec25[(self.IOTA0 & 16383) as usize] = (fTemp84 + (1.0 - fTemp86)) * (fTemp87 * (fTemp88 * (0.041666668 * self.fVec24[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, std::cmp::max(0, iTemp90)))) & 1023) as usize] * fTemp91 - 0.16666667 * fTemp92 * self.fVec24[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, std::cmp::max(0, i32::wrapping_add(iTemp90, 1))))) & 1023) as usize]) + 0.25 * fTemp93 * self.fVec24[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, std::cmp::max(0, i32::wrapping_add(iTemp90, 2))))) & 1023) as usize]) - 0.16666667 * fTemp94 * self.fVec24[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, std::cmp::max(0, i32::wrapping_add(iTemp90, 3))))) & 1023) as usize]) + 0.041666668 * fTemp94 * fTemp87 * self.fVec24[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, std::cmp::max(0, i32::wrapping_add(iTemp90, 4))))) & 1023) as usize];
			self.fRec70[0] = 0.995 * (self.fRec70[1] + (i32::wrapping_mul(iTemp0, iSlow42)) as F32) + fSlow43;
			let mut fTemp95: F32 = self.fRec70[0] + -1.49999;
			let mut fTemp96: F32 = self.fVec25[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp95) as i32)))) & 16383) as usize];
			self.fVec26[0] = fTemp96;
			let mut fTemp97: F32 = F32::floor(fTemp95);
			self.fRec69[0] = self.fVec26[1] - (fTemp97 + (2.0 - self.fRec70[0])) * (self.fRec69[1] - fTemp96) / (self.fRec70[0] - fTemp97);
			let mut fTemp98: F32 = 0.7602446 * self.fRec69[0];
			self.fVec27[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp82 + 0.70710677 * (fTemp83 - fTemp98);
			self.fRec71[0] = 0.9999 * (self.fRec71[1] + (i32::wrapping_mul(iTemp0, iSlow44)) as F32) + fSlow45;
			let mut fTemp99: F32 = self.fRec71[0] + -1.49999;
			let mut fTemp100: F32 = self.fVec27[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp99) as i32)))) & 16383) as usize];
			self.fVec28[0] = fTemp100;
			let mut fTemp101: F32 = F32::floor(fTemp99);
			self.fRec36[0] = self.fVec28[1] - (fTemp101 + (2.0 - self.fRec71[0])) * (self.fRec36[1] - fTemp100) / (self.fRec71[0] - fTemp101);
			self.fRec34[0] = self.fRec36[0];
			self.fVec29[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp82 + 0.70710677 * (fTemp98 - fTemp83);
			self.fRec73[0] = 0.9999 * (self.fRec73[1] + (i32::wrapping_mul(iTemp0, iSlow46)) as F32) + fSlow47;
			let mut fTemp102: F32 = self.fRec73[0] + -1.49999;
			let mut fTemp103: F32 = self.fVec29[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp102) as i32)))) & 16383) as usize];
			self.fVec30[0] = fTemp103;
			let mut fTemp104: F32 = F32::floor(fTemp102);
			self.fRec72[0] = self.fVec30[1] - (fTemp104 + (2.0 - self.fRec73[0])) * (self.fRec72[1] - fTemp103) / (self.fRec73[0] - fTemp104);
			self.fRec35[0] = self.fRec72[0];
			let mut fTemp105: F32 = 0.7602446 * self.fRec34[1] + 0.6496369 * self.fRec37[0];
			let mut fTemp106: F32 = 0.7602446 * fTemp105 - 0.6496369 * self.fRec31[1];
			let mut fTemp107: F32 = 0.6496369 * self.fRec32[1];
			let mut fTemp108: F32 = 0.7602446 * self.fRec35[1] + 0.6496369 * self.fRec69[0];
			let mut fTemp109: F32 = 0.7602446 * fTemp108;
			self.fVec31[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp106 + 0.70710677 * (fTemp107 - fTemp109);
			self.fRec74[0] = 0.9999 * (self.fRec74[1] + (i32::wrapping_mul(iTemp0, iSlow48)) as F32) + fSlow49;
			let mut fTemp110: F32 = self.fRec74[0] + -1.49999;
			let mut fTemp111: F32 = self.fVec31[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp110) as i32)))) & 16383) as usize];
			self.fVec32[0] = fTemp111;
			let mut fTemp112: F32 = F32::floor(fTemp110);
			self.fRec33[0] = self.fVec32[1] - (fTemp112 + (2.0 - self.fRec74[0])) * (self.fRec33[1] - fTemp111) / (self.fRec74[0] - fTemp112);
			self.fRec31[0] = self.fRec33[0];
			self.fVec33[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp106 + 0.70710677 * (fTemp109 - fTemp107);
			self.fRec76[0] = 0.9999 * (self.fRec76[1] + (i32::wrapping_mul(iTemp0, iSlow50)) as F32) + fSlow51;
			let mut fTemp113: F32 = self.fRec76[0] + -1.49999;
			let mut iTemp114: i32 = std::cmp::min(8192, std::cmp::max(0, (fTemp113) as i32));
			let mut fTemp115: F32 = self.fVec33[((i32::wrapping_sub(self.IOTA0, iTemp114)) & 16383) as usize];
			self.fVec34[0] = fTemp115;
			let mut fTemp116: F32 = F32::floor(fTemp113);
			let mut fTemp117: F32 = fTemp116 + (2.0 - self.fRec76[0]);
			let mut fTemp118: F32 = self.fRec76[0] - fTemp116;
			self.fRec75[0] = self.fVec34[1] - fTemp117 * (self.fRec75[1] - fTemp115) / fTemp118;
			self.fRec32[0] = self.fRec75[0];
			let mut fTemp119: F32 = 0.7602446 * self.fRec31[1] + 0.6496369 * fTemp105;
			let mut fTemp120: F32 = 0.7602446 * fTemp119 - 0.6496369 * self.fRec28[1];
			let mut fTemp121: F32 = 0.6496369 * self.fRec29[1];
			let mut fTemp122: F32 = 0.7602446 * self.fRec32[1] + 0.6496369 * fTemp108;
			let mut fTemp123: F32 = 0.7602446 * fTemp122;
			self.fVec35[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp120 + 0.70710677 * (fTemp121 - fTemp123);
			self.fRec77[0] = 0.9999 * (self.fRec77[1] + (i32::wrapping_mul(iTemp0, iSlow52)) as F32) + fSlow53;
			let mut fTemp124: F32 = self.fRec77[0] + -1.49999;
			let mut fTemp125: F32 = self.fVec35[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp124) as i32)))) & 16383) as usize];
			self.fVec36[0] = fTemp125;
			let mut fTemp126: F32 = F32::floor(fTemp124);
			self.fRec30[0] = self.fVec36[1] - (fTemp126 + (2.0 - self.fRec77[0])) * (self.fRec30[1] - fTemp125) / (self.fRec77[0] - fTemp126);
			self.fRec28[0] = self.fRec30[0];
			self.fVec37[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp120 + 0.70710677 * (fTemp123 - fTemp121);
			self.fRec79[0] = 0.9999 * (self.fRec79[1] + (i32::wrapping_mul(iTemp0, iSlow54)) as F32) + fSlow55;
			let mut fTemp127: F32 = self.fRec79[0] + -1.49999;
			let mut iTemp128: i32 = std::cmp::min(8192, std::cmp::max(0, (fTemp127) as i32));
			let mut fTemp129: F32 = self.fVec37[((i32::wrapping_sub(self.IOTA0, iTemp128)) & 16383) as usize];
			self.fVec38[0] = fTemp129;
			let mut fTemp130: F32 = F32::floor(fTemp127);
			let mut fTemp131: F32 = fTemp130 + (2.0 - self.fRec79[0]);
			let mut fTemp132: F32 = self.fRec79[0] - fTemp130;
			self.fRec78[0] = self.fVec38[1] - fTemp131 * (self.fRec78[1] - fTemp129) / fTemp132;
			self.fRec29[0] = self.fRec78[0];
			let mut fTemp133: F32 = 0.7602446 * self.fRec28[1] + 0.6496369 * fTemp119;
			let mut fTemp134: F32 = 0.7602446 * fTemp133 - 0.6496369 * self.fRec25[1];
			let mut fTemp135: F32 = 0.6496369 * self.fRec26[1];
			let mut fTemp136: F32 = 0.7602446 * self.fRec29[1] + 0.6496369 * fTemp122;
			let mut fTemp137: F32 = 0.7602446 * fTemp136;
			self.fVec39[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp134 + 0.70710677 * (fTemp135 - fTemp137);
			self.fRec80[0] = 0.9999 * (self.fRec80[1] + (i32::wrapping_mul(iTemp0, iSlow56)) as F32) + fSlow57;
			let mut fTemp138: F32 = self.fRec80[0] + -1.49999;
			let mut iTemp139: i32 = std::cmp::min(8192, std::cmp::max(0, (fTemp138) as i32));
			let mut fTemp140: F32 = self.fVec39[((i32::wrapping_sub(self.IOTA0, iTemp139)) & 16383) as usize];
			self.fVec40[0] = fTemp140;
			let mut fTemp141: F32 = F32::floor(fTemp138);
			let mut fTemp142: F32 = fTemp141 + (2.0 - self.fRec80[0]);
			let mut fTemp143: F32 = self.fRec80[0] - fTemp141;
			self.fRec27[0] = self.fVec40[1] - fTemp142 * (self.fRec27[1] - fTemp140) / fTemp143;
			self.fRec25[0] = self.fRec27[0];
			self.fVec41[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp134 + 0.70710677 * (fTemp137 - fTemp135);
			self.fRec82[0] = 0.9999 * (self.fRec82[1] + (i32::wrapping_mul(iTemp0, iSlow58)) as F32) + fSlow59;
			let mut fTemp144: F32 = self.fRec82[0] + -1.49999;
			let mut iTemp145: i32 = std::cmp::min(8192, std::cmp::max(0, (fTemp144) as i32));
			let mut fTemp146: F32 = self.fVec41[((i32::wrapping_sub(self.IOTA0, iTemp145)) & 16383) as usize];
			self.fVec42[0] = fTemp146;
			let mut fTemp147: F32 = F32::floor(fTemp144);
			let mut fTemp148: F32 = fTemp147 + (2.0 - self.fRec82[0]);
			let mut fTemp149: F32 = self.fRec82[0] - fTemp147;
			self.fRec81[0] = self.fVec42[1] - fTemp148 * (self.fRec81[1] - fTemp146) / fTemp149;
			self.fRec26[0] = self.fRec81[0];
			let mut fTemp150: F32 = 0.7602446 * self.fRec25[1] + 0.6496369 * fTemp133;
			let mut fTemp151: F32 = 0.7602446 * fTemp150 - 0.6496369 * self.fRec22[1];
			let mut fTemp152: F32 = 0.6496369 * self.fRec23[1];
			let mut fTemp153: F32 = 0.7602446 * self.fRec26[1] + 0.6496369 * fTemp136;
			let mut fTemp154: F32 = 0.7602446 * fTemp153;
			self.fVec43[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp151 + 0.70710677 * (fTemp152 - fTemp154);
			self.fRec83[0] = 0.9999 * (self.fRec83[1] + (i32::wrapping_mul(iTemp0, iSlow60)) as F32) + fSlow61;
			let mut fTemp155: F32 = self.fRec83[0] + -1.49999;
			let mut fTemp156: F32 = self.fVec43[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp155) as i32)))) & 16383) as usize];
			self.fVec44[0] = fTemp156;
			let mut fTemp157: F32 = F32::floor(fTemp155);
			self.fRec24[0] = self.fVec44[1] - (fTemp157 + (2.0 - self.fRec83[0])) * (self.fRec24[1] - fTemp156) / (self.fRec83[0] - fTemp157);
			self.fRec22[0] = self.fRec24[0];
			self.fVec45[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp151 + 0.70710677 * (fTemp154 - fTemp152);
			self.fRec85[0] = 0.9999 * (self.fRec85[1] + (i32::wrapping_mul(iTemp0, iSlow62)) as F32) + fSlow63;
			let mut fTemp158: F32 = self.fRec85[0] + -1.49999;
			let mut iTemp159: i32 = std::cmp::min(8192, std::cmp::max(0, (fTemp158) as i32));
			let mut fTemp160: F32 = self.fVec45[((i32::wrapping_sub(self.IOTA0, iTemp159)) & 16383) as usize];
			self.fVec46[0] = fTemp160;
			let mut fTemp161: F32 = F32::floor(fTemp158);
			let mut fTemp162: F32 = fTemp161 + (2.0 - self.fRec85[0]);
			let mut fTemp163: F32 = self.fRec85[0] - fTemp161;
			self.fRec84[0] = self.fVec46[1] - fTemp162 * (self.fRec84[1] - fTemp160) / fTemp163;
			self.fRec23[0] = self.fRec84[0];
			let mut fTemp164: F32 = 0.7602446 * self.fRec22[1] + 0.6496369 * fTemp150;
			self.fVec47[(self.IOTA0 & 16383) as usize] = fTemp164;
			let mut iTemp165: i32 = (fTemp2) as i32;
			let mut iTemp166: i32 = std::cmp::max(0, iTemp165);
			let mut fTemp167: F32 = fTemp1 + (5.0 - fTemp3);
			let mut iTemp168: i32 = std::cmp::max(0, i32::wrapping_add(iTemp165, 1));
			let mut fTemp169: F32 = fTemp167 * fTemp7;
			let mut iTemp170: i32 = std::cmp::max(0, i32::wrapping_add(iTemp165, 2));
			let mut fTemp171: F32 = fTemp169 * fTemp6;
			let mut iTemp172: i32 = std::cmp::max(0, i32::wrapping_add(iTemp165, 3));
			let mut fTemp173: F32 = fTemp171 * fTemp5;
			let mut iTemp174: i32 = std::cmp::max(0, i32::wrapping_add(iTemp165, 4));
			self.fVec48[(self.IOTA0 & 16383) as usize] = fTemp4 * (fTemp5 * (fTemp6 * (0.041666668 * self.fVec47[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, iTemp166))) & 16383) as usize] * fTemp7 - 0.16666667 * fTemp167 * self.fVec47[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, iTemp168))) & 16383) as usize]) + 0.25 * fTemp169 * self.fVec47[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, iTemp170))) & 16383) as usize]) - 0.16666667 * fTemp171 * self.fVec47[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, iTemp172))) & 16383) as usize]) + 0.041666668 * fTemp173 * self.fVec47[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, iTemp174))) & 16383) as usize];
			self.fRec86[0] = 0.995 * (self.fRec86[1] + (i32::wrapping_mul(iTemp0, iSlow64)) as F32) + fSlow65;
			let mut fTemp175: F32 = self.fRec86[0] + -1.49999;
			let mut fTemp176: F32 = self.fVec48[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp175) as i32)))) & 16383) as usize];
			self.fVec49[0] = fTemp176;
			let mut fTemp177: F32 = F32::floor(fTemp175);
			self.fRec21[0] = self.fVec49[1] - (fTemp177 + (2.0 - self.fRec86[0])) * (self.fRec21[1] - fTemp176) / (self.fRec86[0] - fTemp177);
			self.fRec20[0] = -(fSlow18 * (fSlow19 * self.fRec20[1] - fSlow13 * (self.fRec21[0] - self.fRec21[1])));
			self.fRec19[0] = self.fRec20[0] - fSlow66 * (fSlow67 * self.fRec19[2] + fSlow68 * self.fRec19[1]);
			self.fRec18[0] = fSlow17 * (self.fRec19[2] + (self.fRec19[0] - 2.0 * self.fRec19[1])) - fSlow69 * (fSlow70 * self.fRec18[2] + fSlow68 * self.fRec18[1]);
			let mut fTemp178: F32 = fSlow77 * self.fRec17[1];
			self.fRec17[0] = fSlow15 * (self.fRec18[2] + (self.fRec18[0] - 2.0 * self.fRec18[1])) - fSlow73 * (fSlow74 * self.fRec17[2] + fTemp178);
			self.fRec92[0] = -(fSlow18 * (fSlow19 * self.fRec92[1] - (self.fRec21[0] + self.fRec21[1])));
			self.fRec91[0] = self.fRec92[0] - fSlow66 * (fSlow67 * self.fRec91[2] + fSlow68 * self.fRec91[1]);
			self.fRec90[0] = fSlow66 * (self.fRec91[2] + self.fRec91[0] + 2.0 * self.fRec91[1]) - fSlow69 * (fSlow70 * self.fRec90[2] + fSlow68 * self.fRec90[1]);
			let mut fTemp179: F32 = fSlow69 * (self.fRec90[2] + self.fRec90[0] + 2.0 * self.fRec90[1]);
			self.fVec50[0] = fTemp179;
			self.fRec89[0] = -(fSlow81 * (fSlow82 * self.fRec89[1] - fSlow72 * (fTemp179 - self.fVec50[1])));
			self.fRec88[0] = self.fRec89[0] - fSlow83 * (fSlow84 * self.fRec88[2] + fSlow77 * self.fRec88[1]);
			self.fRec87[0] = fSlow80 * (self.fRec88[2] + (self.fRec88[0] - 2.0 * self.fRec88[1])) - fSlow78 * (fSlow85 * self.fRec87[2] + fSlow77 * self.fRec87[1]);
			self.fRec95[0] = -(fSlow81 * (fSlow82 * self.fRec95[1] - (fTemp179 + self.fVec50[1])));
			self.fRec94[0] = self.fRec95[0] - fSlow83 * (fSlow84 * self.fRec94[2] + fSlow77 * self.fRec94[1]);
			self.fRec93[0] = fSlow83 * (self.fRec94[2] + self.fRec94[0] + 2.0 * self.fRec94[1]) - fSlow78 * (fSlow85 * self.fRec93[2] + fSlow77 * self.fRec93[1]);
			let mut fTemp180: F32 = *input0 + fSlow9 * (fSlow10 * (self.fRec17[2] + fSlow73 * (fTemp178 + fSlow74 * self.fRec17[0])) + fSlow78 * (fSlow76 * (self.fRec87[2] + (self.fRec87[0] - 2.0 * self.fRec87[1])) + fSlow86 * (self.fRec93[2] + self.fRec93[0] + 2.0 * self.fRec93[1])));
			self.fVec51[(self.IOTA0 & 1023) as usize] = fTemp180;
			self.fRec14[0] = fSlow3 * (fTemp4 * (fTemp5 * (fTemp6 * (0.041666668 * fTemp7 * self.fVec51[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, iTemp166))) & 1023) as usize] - 0.16666667 * fTemp167 * self.fVec51[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, iTemp168))) & 1023) as usize]) + 0.25 * fTemp169 * self.fVec51[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, iTemp170))) & 1023) as usize]) - 0.16666667 * fTemp171 * self.fVec51[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, iTemp172))) & 1023) as usize]) + 0.041666668 * fTemp173 * self.fVec51[((i32::wrapping_sub(self.IOTA0, std::cmp::min(512, iTemp174))) & 1023) as usize]) + fSlow2 * self.fRec14[1];
			let mut fTemp181: F32 = fSlow1 * self.fRec14[0] - fSlow87 * self.fRec11[1];
			let mut fTemp182: F32 = fSlow87 * self.fRec12[1];
			let mut fTemp183: F32 = fSlow4 * (1.0 - self.fRec15[0]);
			let mut fTemp184: F32 = fTemp183 + 3.500005;
			let mut fTemp185: F32 = F32::floor(fTemp184);
			let mut fTemp186: F32 = fTemp183 + (2.0 - fTemp185);
			let mut fTemp187: F32 = fTemp183 + (3.0 - fTemp185);
			let mut fTemp188: F32 = 0.7602446 * self.fRec23[1] + 0.6496369 * fTemp153;
			self.fVec52[(self.IOTA0 & 16383) as usize] = fTemp188;
			let mut iTemp189: i32 = (fTemp184) as i32;
			let mut fTemp190: F32 = fTemp183 + (4.0 - fTemp185);
			let mut fTemp191: F32 = fTemp183 + (5.0 - fTemp185);
			let mut fTemp192: F32 = fTemp191 * fTemp190;
			let mut fTemp193: F32 = fTemp192 * fTemp187;
			self.fVec53[(self.IOTA0 & 16383) as usize] = (fTemp183 + (1.0 - fTemp185)) * (fTemp186 * (fTemp187 * (0.041666668 * self.fVec52[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp189)))) & 16383) as usize] * fTemp190 - 0.16666667 * fTemp191 * self.fVec52[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp189, 1))))) & 16383) as usize]) + 0.25 * fTemp192 * self.fVec52[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp189, 2))))) & 16383) as usize]) - 0.16666667 * fTemp193 * self.fVec52[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp189, 3))))) & 16383) as usize]) + 0.041666668 * fTemp193 * fTemp186 * self.fVec52[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp189, 4))))) & 16383) as usize];
			self.fRec102[0] = 0.995 * (self.fRec102[1] + (i32::wrapping_mul(iTemp0, iSlow88)) as F32) + fSlow89;
			let mut fTemp194: F32 = self.fRec102[0] + -1.49999;
			let mut fTemp195: F32 = self.fVec53[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp194) as i32)))) & 16383) as usize];
			self.fVec54[0] = fTemp195;
			let mut fTemp196: F32 = F32::floor(fTemp194);
			self.fRec101[0] = self.fVec54[1] - (fTemp196 + (2.0 - self.fRec102[0])) * (self.fRec101[1] - fTemp195) / (self.fRec102[0] - fTemp196);
			self.fRec100[0] = -(fSlow18 * (fSlow19 * self.fRec100[1] - fSlow13 * (self.fRec101[0] - self.fRec101[1])));
			self.fRec99[0] = self.fRec100[0] - fSlow66 * (fSlow67 * self.fRec99[2] + fSlow68 * self.fRec99[1]);
			self.fRec98[0] = fSlow17 * (self.fRec99[2] + (self.fRec99[0] - 2.0 * self.fRec99[1])) - fSlow69 * (fSlow70 * self.fRec98[2] + fSlow68 * self.fRec98[1]);
			let mut fTemp197: F32 = fSlow77 * self.fRec97[1];
			self.fRec97[0] = fSlow15 * (self.fRec98[2] + (self.fRec98[0] - 2.0 * self.fRec98[1])) - fSlow73 * (fSlow74 * self.fRec97[2] + fTemp197);
			self.fRec108[0] = -(fSlow18 * (fSlow19 * self.fRec108[1] - (self.fRec101[0] + self.fRec101[1])));
			self.fRec107[0] = self.fRec108[0] - fSlow66 * (fSlow67 * self.fRec107[2] + fSlow68 * self.fRec107[1]);
			self.fRec106[0] = fSlow66 * (self.fRec107[2] + self.fRec107[0] + 2.0 * self.fRec107[1]) - fSlow69 * (fSlow70 * self.fRec106[2] + fSlow68 * self.fRec106[1]);
			let mut fTemp198: F32 = fSlow69 * (self.fRec106[2] + self.fRec106[0] + 2.0 * self.fRec106[1]);
			self.fVec55[0] = fTemp198;
			self.fRec105[0] = -(fSlow81 * (fSlow82 * self.fRec105[1] - fSlow72 * (fTemp198 - self.fVec55[1])));
			self.fRec104[0] = self.fRec105[0] - fSlow83 * (fSlow84 * self.fRec104[2] + fSlow77 * self.fRec104[1]);
			self.fRec103[0] = fSlow80 * (self.fRec104[2] + (self.fRec104[0] - 2.0 * self.fRec104[1])) - fSlow78 * (fSlow85 * self.fRec103[2] + fSlow77 * self.fRec103[1]);
			self.fRec111[0] = -(fSlow81 * (fSlow82 * self.fRec111[1] - (fTemp198 + self.fVec55[1])));
			self.fRec110[0] = self.fRec111[0] - fSlow83 * (fSlow84 * self.fRec110[2] + fSlow77 * self.fRec110[1]);
			self.fRec109[0] = fSlow83 * (self.fRec110[2] + self.fRec110[0] + 2.0 * self.fRec110[1]) - fSlow78 * (fSlow85 * self.fRec109[2] + fSlow77 * self.fRec109[1]);
			let mut fTemp199: F32 = *input1 + fSlow9 * (fSlow10 * (self.fRec97[2] + fSlow73 * (fTemp197 + fSlow74 * self.fRec97[0])) + fSlow78 * (fSlow76 * (self.fRec103[2] + (self.fRec103[0] - 2.0 * self.fRec103[1])) + fSlow86 * (self.fRec109[2] + self.fRec109[0] + 2.0 * self.fRec109[1])));
			self.fVec56[(self.IOTA0 & 1023) as usize] = fTemp199;
			self.fRec96[0] = fSlow3 * (fTemp11 * (fTemp12 * (fTemp13 * (0.041666668 * fTemp70 * self.fVec56[((i32::wrapping_sub(self.IOTA0, iTemp69)) & 1023) as usize] - 0.16666667 * fTemp71 * self.fVec56[((i32::wrapping_sub(self.IOTA0, iTemp72)) & 1023) as usize]) + 0.25 * fTemp73 * self.fVec56[((i32::wrapping_sub(self.IOTA0, iTemp74)) & 1023) as usize]) - 0.16666667 * fTemp75 * self.fVec56[((i32::wrapping_sub(self.IOTA0, iTemp76)) & 1023) as usize]) + 0.041666668 * fTemp77 * self.fVec56[((i32::wrapping_sub(self.IOTA0, iTemp78)) & 1023) as usize]) + fSlow2 * self.fRec96[1];
			let mut fTemp200: F32 = fSlow1 * self.fRec96[0];
			self.fVec57[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp181 + 0.70710677 * (fTemp182 - fTemp200);
			let mut fTemp201: F32 = self.fVec57[((i32::wrapping_sub(self.IOTA0, iTemp114)) & 16383) as usize];
			self.fVec58[0] = fTemp201;
			self.fRec13[0] = self.fVec58[1] - fTemp117 * (self.fRec13[1] - fTemp201) / fTemp118;
			self.fRec11[0] = self.fRec13[0];
			self.fVec59[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp181 + 0.70710677 * (fTemp200 - fTemp182);
			self.fRec113[0] = 0.9999 * (self.fRec113[1] + (i32::wrapping_mul(iTemp0, iSlow90)) as F32) + fSlow91;
			let mut fTemp202: F32 = self.fRec113[0] + -1.49999;
			let mut fTemp203: F32 = self.fVec59[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp202) as i32)))) & 16383) as usize];
			self.fVec60[0] = fTemp203;
			let mut fTemp204: F32 = F32::floor(fTemp202);
			self.fRec112[0] = self.fVec60[1] - (fTemp204 + (2.0 - self.fRec113[0])) * (self.fRec112[1] - fTemp203) / (self.fRec113[0] - fTemp204);
			self.fRec12[0] = self.fRec112[0];
			let mut fTemp205: F32 = fSlow1 * self.fRec11[1] + fSlow87 * self.fRec14[0];
			let mut fTemp206: F32 = fSlow1 * fTemp205 - fSlow87 * self.fRec8[1];
			let mut fTemp207: F32 = fSlow87 * self.fRec9[1];
			let mut fTemp208: F32 = fSlow1 * self.fRec12[1] + fSlow87 * self.fRec96[0];
			let mut fTemp209: F32 = fSlow1 * fTemp208;
			self.fVec61[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp206 + 0.70710677 * (fTemp207 - fTemp209);
			let mut fTemp210: F32 = self.fVec61[((i32::wrapping_sub(self.IOTA0, iTemp139)) & 16383) as usize];
			self.fVec62[0] = fTemp210;
			self.fRec10[0] = self.fVec62[1] - fTemp142 * (self.fRec10[1] - fTemp210) / fTemp143;
			self.fRec8[0] = self.fRec10[0];
			self.fVec63[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp206 + 0.70710677 * (fTemp209 - fTemp207);
			let mut fTemp211: F32 = self.fVec63[((i32::wrapping_sub(self.IOTA0, iTemp128)) & 16383) as usize];
			self.fVec64[0] = fTemp211;
			self.fRec114[0] = self.fVec64[1] - fTemp131 * (self.fRec114[1] - fTemp211) / fTemp132;
			self.fRec9[0] = self.fRec114[0];
			let mut fTemp212: F32 = fSlow1 * self.fRec8[1] + fSlow87 * fTemp205;
			let mut fTemp213: F32 = fSlow1 * fTemp212 - fSlow87 * self.fRec5[1];
			let mut fTemp214: F32 = fSlow87 * self.fRec6[1];
			let mut fTemp215: F32 = fSlow1 * self.fRec9[1] + fSlow87 * fTemp208;
			let mut fTemp216: F32 = fSlow1 * fTemp215;
			self.fVec65[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp213 + 0.70710677 * (fTemp214 - fTemp216);
			let mut fTemp217: F32 = self.fVec65[((i32::wrapping_sub(self.IOTA0, iTemp145)) & 16383) as usize];
			self.fVec66[0] = fTemp217;
			self.fRec7[0] = self.fVec66[1] - fTemp148 * (self.fRec7[1] - fTemp217) / fTemp149;
			self.fRec5[0] = self.fRec7[0];
			self.fVec67[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp213 + 0.70710677 * (fTemp216 - fTemp214);
			self.fRec116[0] = 0.9999 * (self.fRec116[1] + (i32::wrapping_mul(iTemp0, iSlow92)) as F32) + fSlow93;
			let mut fTemp218: F32 = self.fRec116[0] + -1.49999;
			let mut fTemp219: F32 = self.fVec67[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp218) as i32)))) & 16383) as usize];
			self.fVec68[0] = fTemp219;
			let mut fTemp220: F32 = F32::floor(fTemp218);
			self.fRec115[0] = self.fVec68[1] - (fTemp220 + (2.0 - self.fRec116[0])) * (self.fRec115[1] - fTemp219) / (self.fRec116[0] - fTemp220);
			self.fRec6[0] = self.fRec115[0];
			let mut fTemp221: F32 = fSlow1 * self.fRec5[1] + fSlow87 * fTemp212;
			let mut fTemp222: F32 = fSlow1 * fTemp221 - fSlow87 * self.fRec2[1];
			let mut fTemp223: F32 = fSlow87 * self.fRec3[1];
			let mut fTemp224: F32 = fSlow1 * self.fRec6[1] + fSlow87 * fTemp215;
			let mut fTemp225: F32 = fSlow1 * fTemp224;
			self.fVec69[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp222 + 0.70710677 * (fTemp223 - fTemp225);
			self.fRec117[0] = 0.9999 * (self.fRec117[1] + (i32::wrapping_mul(iTemp0, iSlow94)) as F32) + fSlow95;
			let mut fTemp226: F32 = self.fRec117[0] + -1.49999;
			let mut fTemp227: F32 = self.fVec69[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, (fTemp226) as i32)))) & 16383) as usize];
			self.fVec70[0] = fTemp227;
			let mut fTemp228: F32 = F32::floor(fTemp226);
			self.fRec4[0] = self.fVec70[1] - (fTemp228 + (2.0 - self.fRec117[0])) * (self.fRec4[1] - fTemp227) / (self.fRec117[0] - fTemp228);
			self.fRec2[0] = self.fRec4[0];
			self.fVec71[(self.IOTA0 & 16383) as usize] = 0.70710677 * fTemp222 + 0.70710677 * (fTemp225 - fTemp223);
			let mut fTemp229: F32 = self.fVec71[((i32::wrapping_sub(self.IOTA0, iTemp159)) & 16383) as usize];
			self.fVec72[0] = fTemp229;
			self.fRec118[0] = self.fVec72[1] - fTemp162 * (self.fRec118[1] - fTemp229) / fTemp163;
			self.fRec3[0] = self.fRec118[0];
			self.fRec0[0] = fSlow1 * self.fRec2[1] + fSlow87 * fTemp221;
			self.fRec1[0] = fSlow1 * self.fRec3[1] + fSlow87 * fTemp224;
			*output0 = self.fRec0[0];
			*output1 = self.fRec1[0];
			self.iVec0[1] = self.iVec0[0];
			self.fRec15[1] = self.fRec15[0];
			self.fRec16[1] = self.fRec16[0];
			self.IOTA0 = i32::wrapping_add(self.IOTA0, 1);
			self.fRec53[1] = self.fRec53[0];
			self.fVec2[1] = self.fVec2[0];
			self.fRec52[1] = self.fRec52[0];
			self.fRec50[1] = self.fRec50[0];
			self.fRec55[1] = self.fRec55[0];
			self.fVec4[1] = self.fVec4[0];
			self.fRec54[1] = self.fRec54[0];
			self.fRec51[1] = self.fRec51[0];
			self.fRec56[1] = self.fRec56[0];
			self.fVec6[1] = self.fVec6[0];
			self.fRec49[1] = self.fRec49[0];
			self.fRec47[1] = self.fRec47[0];
			self.fRec58[1] = self.fRec58[0];
			self.fVec8[1] = self.fVec8[0];
			self.fRec57[1] = self.fRec57[0];
			self.fRec48[1] = self.fRec48[0];
			self.fRec59[1] = self.fRec59[0];
			self.fVec10[1] = self.fVec10[0];
			self.fRec46[1] = self.fRec46[0];
			self.fRec44[1] = self.fRec44[0];
			self.fRec61[1] = self.fRec61[0];
			self.fVec12[1] = self.fVec12[0];
			self.fRec60[1] = self.fRec60[0];
			self.fRec45[1] = self.fRec45[0];
			self.fRec62[1] = self.fRec62[0];
			self.fVec14[1] = self.fVec14[0];
			self.fRec43[1] = self.fRec43[0];
			self.fRec41[1] = self.fRec41[0];
			self.fRec64[1] = self.fRec64[0];
			self.fVec16[1] = self.fVec16[0];
			self.fRec63[1] = self.fRec63[0];
			self.fRec42[1] = self.fRec42[0];
			self.fRec65[1] = self.fRec65[0];
			self.fVec18[1] = self.fVec18[0];
			self.fRec40[1] = self.fRec40[0];
			self.fRec38[1] = self.fRec38[0];
			self.fRec67[1] = self.fRec67[0];
			self.fVec20[1] = self.fVec20[0];
			self.fRec66[1] = self.fRec66[0];
			self.fRec39[1] = self.fRec39[0];
			self.fRec68[1] = self.fRec68[0];
			self.fVec23[1] = self.fVec23[0];
			self.fRec37[1] = self.fRec37[0];
			self.fRec70[1] = self.fRec70[0];
			self.fVec26[1] = self.fVec26[0];
			self.fRec69[1] = self.fRec69[0];
			self.fRec71[1] = self.fRec71[0];
			self.fVec28[1] = self.fVec28[0];
			self.fRec36[1] = self.fRec36[0];
			self.fRec34[1] = self.fRec34[0];
			self.fRec73[1] = self.fRec73[0];
			self.fVec30[1] = self.fVec30[0];
			self.fRec72[1] = self.fRec72[0];
			self.fRec35[1] = self.fRec35[0];
			self.fRec74[1] = self.fRec74[0];
			self.fVec32[1] = self.fVec32[0];
			self.fRec33[1] = self.fRec33[0];
			self.fRec31[1] = self.fRec31[0];
			self.fRec76[1] = self.fRec76[0];
			self.fVec34[1] = self.fVec34[0];
			self.fRec75[1] = self.fRec75[0];
			self.fRec32[1] = self.fRec32[0];
			self.fRec77[1] = self.fRec77[0];
			self.fVec36[1] = self.fVec36[0];
			self.fRec30[1] = self.fRec30[0];
			self.fRec28[1] = self.fRec28[0];
			self.fRec79[1] = self.fRec79[0];
			self.fVec38[1] = self.fVec38[0];
			self.fRec78[1] = self.fRec78[0];
			self.fRec29[1] = self.fRec29[0];
			self.fRec80[1] = self.fRec80[0];
			self.fVec40[1] = self.fVec40[0];
			self.fRec27[1] = self.fRec27[0];
			self.fRec25[1] = self.fRec25[0];
			self.fRec82[1] = self.fRec82[0];
			self.fVec42[1] = self.fVec42[0];
			self.fRec81[1] = self.fRec81[0];
			self.fRec26[1] = self.fRec26[0];
			self.fRec83[1] = self.fRec83[0];
			self.fVec44[1] = self.fVec44[0];
			self.fRec24[1] = self.fRec24[0];
			self.fRec22[1] = self.fRec22[0];
			self.fRec85[1] = self.fRec85[0];
			self.fVec46[1] = self.fVec46[0];
			self.fRec84[1] = self.fRec84[0];
			self.fRec23[1] = self.fRec23[0];
			self.fRec86[1] = self.fRec86[0];
			self.fVec49[1] = self.fVec49[0];
			self.fRec21[1] = self.fRec21[0];
			self.fRec20[1] = self.fRec20[0];
			self.fRec19[2] = self.fRec19[1];
			self.fRec19[1] = self.fRec19[0];
			self.fRec18[2] = self.fRec18[1];
			self.fRec18[1] = self.fRec18[0];
			self.fRec17[2] = self.fRec17[1];
			self.fRec17[1] = self.fRec17[0];
			self.fRec92[1] = self.fRec92[0];
			self.fRec91[2] = self.fRec91[1];
			self.fRec91[1] = self.fRec91[0];
			self.fRec90[2] = self.fRec90[1];
			self.fRec90[1] = self.fRec90[0];
			self.fVec50[1] = self.fVec50[0];
			self.fRec89[1] = self.fRec89[0];
			self.fRec88[2] = self.fRec88[1];
			self.fRec88[1] = self.fRec88[0];
			self.fRec87[2] = self.fRec87[1];
			self.fRec87[1] = self.fRec87[0];
			self.fRec95[1] = self.fRec95[0];
			self.fRec94[2] = self.fRec94[1];
			self.fRec94[1] = self.fRec94[0];
			self.fRec93[2] = self.fRec93[1];
			self.fRec93[1] = self.fRec93[0];
			self.fRec14[1] = self.fRec14[0];
			self.fRec102[1] = self.fRec102[0];
			self.fVec54[1] = self.fVec54[0];
			self.fRec101[1] = self.fRec101[0];
			self.fRec100[1] = self.fRec100[0];
			self.fRec99[2] = self.fRec99[1];
			self.fRec99[1] = self.fRec99[0];
			self.fRec98[2] = self.fRec98[1];
			self.fRec98[1] = self.fRec98[0];
			self.fRec97[2] = self.fRec97[1];
			self.fRec97[1] = self.fRec97[0];
			self.fRec108[1] = self.fRec108[0];
			self.fRec107[2] = self.fRec107[1];
			self.fRec107[1] = self.fRec107[0];
			self.fRec106[2] = self.fRec106[1];
			self.fRec106[1] = self.fRec106[0];
			self.fVec55[1] = self.fVec55[0];
			self.fRec105[1] = self.fRec105[0];
			self.fRec104[2] = self.fRec104[1];
			self.fRec104[1] = self.fRec104[0];
			self.fRec103[2] = self.fRec103[1];
			self.fRec103[1] = self.fRec103[0];
			self.fRec111[1] = self.fRec111[0];
			self.fRec110[2] = self.fRec110[1];
			self.fRec110[1] = self.fRec110[0];
			self.fRec109[2] = self.fRec109[1];
			self.fRec109[1] = self.fRec109[0];
			self.fRec96[1] = self.fRec96[0];
			self.fVec58[1] = self.fVec58[0];
			self.fRec13[1] = self.fRec13[0];
			self.fRec11[1] = self.fRec11[0];
			self.fRec113[1] = self.fRec113[0];
			self.fVec60[1] = self.fVec60[0];
			self.fRec112[1] = self.fRec112[0];
			self.fRec12[1] = self.fRec12[0];
			self.fVec62[1] = self.fVec62[0];
			self.fRec10[1] = self.fRec10[0];
			self.fRec8[1] = self.fRec8[0];
			self.fVec64[1] = self.fVec64[0];
			self.fRec114[1] = self.fRec114[0];
			self.fRec9[1] = self.fRec9[0];
			self.fVec66[1] = self.fVec66[0];
			self.fRec7[1] = self.fRec7[0];
			self.fRec5[1] = self.fRec5[0];
			self.fRec116[1] = self.fRec116[0];
			self.fVec68[1] = self.fVec68[0];
			self.fRec115[1] = self.fRec115[0];
			self.fRec6[1] = self.fRec6[0];
			self.fRec117[1] = self.fRec117[0];
			self.fVec70[1] = self.fVec70[0];
			self.fRec4[1] = self.fRec4[0];
			self.fRec2[1] = self.fRec2[0];
			self.fVec72[1] = self.fVec72[0];
			self.fRec118[1] = self.fRec118[0];
			self.fRec3[1] = self.fRec3[0];
			self.fRec0[1] = self.fRec0[0];
			self.fRec1[1] = self.fRec1[0];
		}
		
	}

}

impl FaustDsp for JpverbDsp {
	type T = FaustFloat;
	fn new() -> Self where Self: Sized {
		Self::new()
	}
	fn metadata(&self, m: &mut dyn Meta) {
		self.metadata(m)
	}
	fn get_sample_rate(&self) -> i32 {
		self.get_sample_rate()
	}
	fn get_num_inputs(&self) -> i32 {
		FAUST_INPUTS as i32
	}
	fn get_num_outputs(&self) -> i32 {
		FAUST_OUTPUTS as i32
	}
	fn class_init(sample_rate: i32) where Self: Sized {
		Self::class_init(sample_rate);
	}
	fn instance_reset_params(&mut self) {
		self.instance_reset_params()
	}
	fn instance_clear(&mut self) {
		self.instance_clear()
	}
	fn instance_constants(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate)
	}
	fn instance_init(&mut self, sample_rate: i32) {
		self.instance_init(sample_rate)
	}
	fn init(&mut self, sample_rate: i32) {
		self.init(sample_rate)
	}
	fn build_user_interface(&self, ui_interface: &mut dyn UI<Self::T>) {
		self.build_user_interface(ui_interface)
	}
	fn build_user_interface_static(ui_interface: &mut dyn UI<Self::T>) where Self: Sized {
		Self::build_user_interface_static(ui_interface);
	}
	fn get_param(&self, param: ParamIndex) -> Option<Self::T> {
		self.get_param(param)
	}
	fn set_param(&mut self, param: ParamIndex, value: Self::T) {
		self.set_param(param, value)
	}
	fn compute(&mut self, count: i32, inputs: &[&[Self::T]], outputs: &mut [&mut [Self::T]]) {
		self.compute(count as usize, inputs, outputs)
	}
}
