import fittings, ducts

cap1 = fittings.OneWayFitting("cap1")
cap2 = fittings.OneWayFitting("cap2")

ducttype1 = ducts.RigidDuctType("ductype1", "rectangular", 0.00009, None, 1, 1)
duct1 = ducts.RigidDuct("duct1", ducttype1, 10, 5)

cap1.connect(duct1, cap2)
# duct with two caps, awesome :D