//
//  FPOpenGLView.h
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  For license see LICENSE.TXT
//

#import <Cocoa/Cocoa.h>
#import "NSOpenGLView+Helpers.h"
#import "FPFont.h"
#import "FPGame.h"
#import "FPReplay.h"

@interface FPGameView : NSOpenGLView
{
	NSTimer *timer;
	FPGame *game;
	NSMutableSet *pressedKeys;
    FPReplay *replay;
    BOOL replayingGame;
}

@property (readwrite, retain) FPGame *game;

- (void)removeAllPressedKeys;
- (void)gameLoop;
- (void)playWithKeyboard;
- (void)playFromRecord;

@end
