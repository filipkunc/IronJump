//
//  FPOpenGLView.h
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
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
