data CTree a b = CT b [CTree a b]
               | CK a
        deriving (Show, Eq)

data Tree a b = T b (Tree a b) (Tree a b)
              | K (Maybe a)
        deriving (Show, Eq)

type NodeType = Int

cons_node :: NodeType
cons_node = -1

uncons :: CTree a NodeType -> Tree a NodeType
uncons (CK a) = K $ Just a
uncons (CT u (x:[])) = T u (uncons x) $ K Nothing
uncons (CT u (x:xs)) = T u (uncons x) $ uncons $ CT cons_node xs
uncons _ = error "node with no children"

recons :: Tree a NodeType -> CTree a NodeType
recons (K (Just a)) = CK a
-- only one child
recons (T u left (K Nothing)) = CT u [recons left]
-- two children - right one is cons
recons (T u left right) = CT u $ (:) (recons left) $ recons' right
                        where
                            recons' (K Nothing) = []
                            recons' (T u left right)
                              | u == cons_node = (:) (recons left) $ recons' right
                              | otherwise = error "non-cons right child"
recons _ = error "empty constant"

size :: Tree a b -> Int
size (K Nothing) = 0
size (K _) = 1
size (T _ left right) = 1 + size left + size right

data Diff a b = Eps
              | Idt b (Diff a b) (Diff a b)
              | Mod (Tree a b) (Tree a b)
              | ModT (Diff a b) (Diff a b) b b
              | AddL b (Tree a b) (Diff a b)
              | AddR b (Diff a b) (Tree a b)
              | DelL (Diff a b)
              | DelR (Diff a b)
       deriving (Show, Eq)

w :: Diff a b -> Int
w Eps = 0
w (Idt _ x y) = w x + w y
w (AddR _ d t) = 1 + w d + size t
w (AddL _ t d) = 1 + w d + size t
w (DelR d) = 1 + w d
w (DelL d) = 1 + w d
w (Mod x y) = 1 + size x + size y
w (ModT dl dr _ _) = 1 + w dl + w dr

minw :: [Diff a b] -> Diff a b
minw [] = error "empty list"
minw (x:xs) = minw' (x:xs) (w x) x
       where minw' (x:[]) w0 x0 = if w x > w0 then x0 else x
             minw' (x:xs) w0 x0 = let wx = w x
                                  in if w x > w0 then minw' xs w0 x0 else minw' xs wx x

diff :: (Eq a, Eq b) => Tree a b -> Tree a b -> Diff a b
diff (K x) (K y)
  | x == y = Eps
  | otherwise = Mod (K x) (K y)
diff (T a x y) (T b x' y')
  | a == b = let di = Idt a (diff x x') (diff y y')
                 dal = AddL b x' (diff (T a x y) y')
                 dar = AddR b (diff (T a x y) x') y'
                 ddl = DelL (diff y (T b x' y'))
                 ddr = DelR (diff x (T b x' y'))
             in minw [di, dal, dar, ddl, ddr]
  | otherwise = let dm  = Mod (T a x y) (T b x' y')
                    dmt = ModT (diff x x') (diff y y') a b
                    dal = AddL b x' (diff (T a x y) y')
                    dar = AddR b (diff (T a x y) x') y'
                    ddl = DelL (diff y (T b x' y'))
                    ddr = DelR (diff x (T b x' y'))
                in minw [dm, dmt, dal, dar, ddl, ddr]
diff (K a) (T t x y) = let dm  = Mod (K a) (T t x y)
                           dal = AddL t x (diff (K a) y)
                           dar = AddR t (diff (K a) x) y
                       in minw [dm, dal, dar]
diff (T t x y) (K a) = let dm  = Mod (T t x y) (K a)
                           ddl = DelL (diff y (K a))
                           ddr = DelR (diff x (K a))
                       in minw [dm, ddl, ddr]

patch :: (Eq a, Eq b) => Tree a b -> Diff a b -> Tree a b
patch x Eps = x
patch x (Mod a b)
  | x == a = b
  | otherwise = error "p(x, mod(a, b)) where x /= a"
patch (T t x y) (Idt t' dx dy)
  | t == t' = T t (patch x dx) (patch y dy)
  | otherwise = error "p(t, Idt) where type(t) /= type(Idt"
patch x (AddL t x' dy) = T t x' (patch x dy)
patch x (AddR t dx y') = T t (patch x dx) y'
patch (T _ _ y) (DelL dy) = patch y dy
patch (T _ x _) (DelR dx) = patch x dx
patch (T t x y) (ModT dx dy t0 t1)
  | t0 == t = T t1 (patch x dx) (patch y dy)
  | otherwise = error "patch(t, modT(t0, t1)) where t /= t0"

-- Tests
testCons :: IO ()
testCons = let t1 = CT 0 [CT 1 [CK "f"], CT 2 [CT 3 [CK "5"], CT 3 [CK "6"]]]
         in do
             putStrLn "==== testCons ===="
             check t1
         where check x = let y = (recons . uncons) x
                         in if (recons . uncons) x == x
                            then putStr "OK: " >> print x
                            else putStr "Fail: left = " >> putStr (show x) >> putStr "; right = " >> print y

testDiff :: IO ()
testDiff = let ct1 = CT 0 [CT 1 [CK "f"], CT 2 [CT 3 [CK "5"], CT 3 [CK "6"]]]
               ct2 = CT 0 [CT 1 [CK "g"], CT 2 [CT 3 [CK "5"], CT 3 [CK "7"]]]
           in let t1 = uncons ct1; t2 = uncons ct2
           in let d = diff t1 t2
           in do
               putStrLn "==== testDiff ===="
               putStr "t1 (cons): "
               print ct1
               putStr "t2 (cons): "
               print ct2
               putStr "t1 (tree): "
               print t1
               putStr "t2 (tree): "
               print t2
               putStr "diff t1 t2 = "
               print d

testPatch :: IO ()
testPatch = let ct1 = CT 0 [CT 1 [CK "f"], CT 2 [CT 3 [CK "5"], CT 3 [CK "6"]]]
                ct2 = CT 0 [CT 1 [CK "g"], CT 2 [CT 3 [CK "5"], CT 3 [CK "7"]]]
           in let t1 = uncons ct1; t2 = uncons ct2
           in let d = diff t1 t2
           in do
               putStrLn "==== testPatch ===="
               putStr "t1 (cons): "
               print ct1
               putStr "t2 (cons): "
               print ct2
               putStr "t1 (tree): "
               print t1
               putStr "t2 (tree): "
               print t2
               putStr "diff t1 t2 = "
               print d
               putStr "patch t1 (diff t1 t2) = "
               print $ patch t1 d
               putStrLn "-- p t1 (d t1 t2) = t2 ?"
               print $ patch t1 d == t2

main :: IO ()
main = testCons >> testDiff >> testPatch
